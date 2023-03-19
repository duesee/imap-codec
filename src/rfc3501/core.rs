use std::{borrow::Cow, convert::TryFrom, num::NonZeroU32, str::from_utf8};

use abnf_core::streaming::{is_ALPHA, is_CHAR, is_CTL, is_DIGIT, CRLF, DQUOTE};
use base64::{engine::general_purpose::STANDARD as _base64, Engine};
use imap_types::{
    core::{AString, Atom, AtomExt, IString, Literal, NString, Quoted},
    message::{Charset, Tag},
    response::{data::QuotedChar, Text},
};
use nom::{
    branch::alt,
    bytes::streaming::{escaped, tag, tag_no_case, take, take_while, take_while1, take_while_m_n},
    character::streaming::{digit1, one_of},
    combinator::{map, map_res, opt, recognize},
    error::ErrorKind,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use crate::{rfc3501::mailbox::is_list_wildcards, utils::unescape_quoted};

// ----- number -----

/// `number = 1*DIGIT`
///
/// Unsigned 32-bit integer (0 <= n < 4,294,967,296)
pub fn number(input: &[u8]) -> IResult<&[u8], u32> {
    map_res(
        // # Safety
        //
        // `unwrap` is safe because `1*DIGIT` contains ASCII-only characters.
        map(digit1, |val| from_utf8(val).unwrap()),
        str::parse::<u32>,
    )(input)
}

/// ```abnf
/// number64 = 1*DIGIT
/// ```
///
/// Unsigned 63-bit integer (0 <= n <= 9,223,372,036,854,775,807)
///
/// Defined in RFC 9051
#[cfg(feature = "ext_quota")]
pub fn number64(input: &[u8]) -> IResult<&[u8], u64> {
    map_res(
        // # Safety
        //
        // `unwrap` is safe because `1*DIGIT` contains ASCII-only characters.
        map(digit1, |val| from_utf8(val).unwrap()),
        str::parse::<u64>,
    )(input)
}

/// `nz-number = digit-nz *DIGIT`
///
/// Non-zero unsigned 32-bit integer (0 < n < 4,294,967,296)
pub fn nz_number(input: &[u8]) -> IResult<&[u8], NonZeroU32> {
    let (remaining, number) = number(input)?;

    match NonZeroU32::new(number) {
        Some(number) => Ok((remaining, number)),
        None => Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::Verify,
        ))),
    }
}

// 1-9
//
// digit-nz = %x31-39
// fn is_digit_nz(byte: u8) -> bool {
//     matches!(byte, b'1'..=b'9')
// }

// ----- string -----

/// `string = quoted / literal`
pub fn string(input: &[u8]) -> IResult<&[u8], IString> {
    alt((map(quoted, IString::Quoted), map(literal, IString::Literal)))(input)
}

/// `quoted = DQUOTE *QUOTED-CHAR DQUOTE`
///
/// This function only allocates a new String, when needed, i.e. when
/// quoted chars need to be replaced.
pub fn quoted(input: &[u8]) -> IResult<&[u8], Quoted> {
    let mut parser = tuple((
        DQUOTE,
        map(
            escaped(
                take_while1(is_any_text_char_except_quoted_specials),
                '\\',
                one_of("\\\""),
            ),
            // # Saftey
            //
            // `unwrap` is safe because val contains ASCII-only characters.
            |val| from_utf8(val).unwrap(),
        ),
        DQUOTE,
    ));

    let (remaining, (_, quoted, _)) = parser(input)?;

    Ok((remaining, Quoted::new_unchecked(unescape_quoted(quoted))))
}

/// `QUOTED-CHAR = <any TEXT-CHAR except quoted-specials> / "\" quoted-specials`
pub fn quoted_char(input: &[u8]) -> IResult<&[u8], QuotedChar> {
    map(
        alt((
            map(
                take_while_m_n(1, 1, is_any_text_char_except_quoted_specials),
                |bytes: &[u8]| {
                    assert_eq!(bytes.len(), 1);
                    bytes[0] as char
                },
            ),
            map(
                tuple((tag("\\"), take_while_m_n(1, 1, is_quoted_specials))),
                |(_, bytes): (_, &[u8])| {
                    assert_eq!(bytes.len(), 1);
                    bytes[0] as char
                },
            ),
        )),
        QuotedChar::new_unchecked,
    )(input)
}

pub(crate) fn is_any_text_char_except_quoted_specials(byte: u8) -> bool {
    is_text_char(byte) && !is_quoted_specials(byte)
}

/// `quoted-specials = DQUOTE / "\"`
pub fn is_quoted_specials(byte: u8) -> bool {
    byte == b'"' || byte == b'\\'
}

/// `literal = "{" number "}" CRLF *CHAR8`
///
/// Number represents the number of CHAR8s
pub fn literal(input: &[u8]) -> IResult<&[u8], Literal> {
    let (remaining, number) = terminated(delimited(tag(b"{"), number, tag(b"}")), CRLF)(input)?;

    // TODO(#40)
    // Signal that an continuation request is required.
    // There are some issues with this ...
    //   * The return type is ad-hoc and does not tell *how* many bytes are about to be send
    //   * It doesn't capture the case when there is something in the buffer already.
    //     This is basically good for us, but there could be issues with servers violating the
    //     IMAP protocol and sending data right away.
    if remaining.is_empty() {
        return Err(nom::Err::Failure(nom::error::Error::new(
            remaining,
            ErrorKind::Fix,
        )));
    }

    let (remaining, data) = take(number)(remaining)?;

    match Literal::try_from(data) {
        Ok(literal) => Ok((remaining, literal)),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            remaining,
            ErrorKind::Verify,
        ))),
    }
}

#[inline]
/// `CHAR8 = %x01-ff`
///
/// Any OCTET except NUL, %x00
// pub fn is_char8(i: u8) -> bool {
//     i != 0
// }

// ----- astring ----- atom (roughly) or string

/// `astring = 1*ASTRING-CHAR / string`
pub fn astring(input: &[u8]) -> IResult<&[u8], AString> {
    alt((
        map(take_while1(is_astring_char), |bytes: &[u8]| {
            // # Safety
            //
            // `unwrap` is safe, because `is_astring_char` enforces that the bytes ...
            //   * contain ASCII-only characters, i.e., `from_utf8` will return `Ok`.
            //   * are valid according to `AtomExt::verify(), i.e., `new_unchecked` is safe.
            AString::Atom(AtomExt::new_unchecked(Cow::Borrowed(
                std::str::from_utf8(bytes).unwrap(),
            )))
        }),
        map(string, AString::String),
    ))(input)
}

/// `ASTRING-CHAR = ATOM-CHAR / resp-specials`
pub fn is_astring_char(i: u8) -> bool {
    is_atom_char(i) || is_resp_specials(i)
}

/// `ATOM-CHAR = <any CHAR except atom-specials>`
pub fn is_atom_char(b: u8) -> bool {
    is_CHAR(b) && !is_atom_specials(b)
}

/// `atom-specials = "(" / ")" / "{" / SP / CTL / list-wildcards / quoted-specials / resp-specials`
pub fn is_atom_specials(i: u8) -> bool {
    match i {
        b'(' | b')' | b'{' | b' ' => true,
        c if is_CTL(c) => true,
        c if is_list_wildcards(c) => true,
        c if is_quoted_specials(c) => true,
        c if is_resp_specials(c) => true,
        _ => false,
    }
}

#[inline]
/// `resp-specials = "]"`
pub fn is_resp_specials(i: u8) -> bool {
    i == b']'
}

/// `atom = 1*ATOM-CHAR`
pub fn atom(input: &[u8]) -> IResult<&[u8], Atom> {
    let parser = take_while1(is_atom_char);

    let (remaining, parsed_atom) = parser(input)?;

    // # Saftey
    //
    // `unwrap` is safe, because `is_atom_char` enforces ...
    // * that the string is always UTF8, and ...
    // * contains only the allowed characters.
    Ok((
        remaining,
        Atom::new_unchecked(Cow::Borrowed(std::str::from_utf8(parsed_atom).unwrap())),
    ))
}

// ----- nstring ----- nil or string

/// `nstring = string / nil`
pub fn nstring(input: &[u8]) -> IResult<&[u8], NString> {
    alt((
        map(string, |item| NString(Some(item))),
        map(nil, |_| NString(None)),
    ))(input)
}

#[inline]
/// `nil = "NIL"`
pub fn nil(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag_no_case(b"NIL")(input)
}

// ----- text -----

/// `text = 1*TEXT-CHAR`
pub fn text(input: &[u8]) -> IResult<&[u8], Text> {
    map(take_while1(is_text_char), |bytes|
        // # Safety
        // 
        // `is_text_char` makes sure that the sequence of bytes
        // is always valid ASCII. Thus, it is also valid UTF-8.
        Text::new_unchecked(Cow::Borrowed(std::str::from_utf8(bytes).unwrap())))(input)
}

/// `TEXT-CHAR = %x01-09 / %x0B-0C / %x0E-7F`
///
/// Note: This was `<any CHAR except CR and LF>` before.
///
/// Note: We also exclude `[` and `]` due to the possibility to misuse this as a `Code`.
pub fn is_text_char(c: u8) -> bool {
    matches!(c, 0x01..=0x09 | 0x0b..=0x0c | 0x0e..=0x5a | 0x5c | 0x5e..=0x7f)
}

// ----- base64 -----

/// `base64 = *(4base64-char) [base64-terminal]`
pub fn base64(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    map_res(
        recognize(tuple((
            take_while(is_base64_char),
            opt(alt((tag("=="), tag("=")))),
        ))),
        |input| _base64.decode(input),
    )(input)
}

/// `base64-char = ALPHA / DIGIT / "+" / "/" ; Case-sensitive`
pub fn is_base64_char(i: u8) -> bool {
    is_ALPHA(i) || is_DIGIT(i) || i == b'+' || i == b'/'
}

// base64-terminal = (2base64-char "==") / (3base64-char "=")

// ----- charset -----

/// `charset = atom / quoted`
///
/// Note: see errata id: 261
pub fn charset(input: &[u8]) -> IResult<&[u8], Charset> {
    alt((map(atom, Charset::Atom), map(quoted, Charset::Quoted)))(input)
}

// ----- tag -----

/// `tag = 1*<any ASTRING-CHAR except "+">`
pub fn tag_imap(input: &[u8]) -> IResult<&[u8], Tag> {
    map(
        map(take_while1(|b| is_astring_char(b) && b != b'+'), |val| {
            // # Safety
            //
            // `is_astring_char` ensures that `val` is UTF-8.
            from_utf8(val).unwrap()
        }),
        |val| Tag::new_unchecked(Cow::Borrowed(val)),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use imap_types::core::Quoted;

    use super::*;

    #[test]
    fn test_atom() {
        assert!(atom(b" ").is_err());
        assert!(atom(b"").is_err());

        let (rem, val) = atom(b"a(").unwrap();
        assert_eq!(val, "a".try_into().unwrap());
        assert_eq!(rem, b"(");

        let (rem, val) = atom(b"xxx yyy").unwrap();
        assert_eq!(val, "xxx".try_into().unwrap());
        assert_eq!(rem, b" yyy");
    }

    #[test]
    fn test_quoted() {
        let (rem, val) = quoted(br#""Hello"???"#).unwrap();
        assert_eq!(rem, b"???");
        assert_eq!(val, Quoted::try_from("Hello").unwrap());

        // Allowed escapes...
        assert!(quoted(br#""Hello \" "???"#).is_ok());
        assert!(quoted(br#""Hello \\ "???"#).is_ok());

        // Not allowed escapes...
        assert!(quoted(br#""Hello \a "???"#).is_err());
        assert!(quoted(br#""Hello \z "???"#).is_err());
        assert!(quoted(br#""Hello \? "???"#).is_err());

        let (rem, val) = quoted(br#""Hello \"World\""???"#).unwrap();
        assert_eq!(rem, br#"???"#);
        // Should it be this (Hello \"World\") ...
        //assert_eq!(val, r#"Hello \"World\""#);
        // ... or this (Hello "World")?
        assert_eq!(val, Quoted::try_from("Hello \"World\"").unwrap());

        // Test Incomplete
        assert!(matches!(quoted(br#""#), Err(nom::Err::Incomplete(_))));
        assert!(matches!(quoted(br#""\"#), Err(nom::Err::Incomplete(_))));
        assert!(matches!(
            quoted(br#""Hello "#),
            Err(nom::Err::Incomplete(_))
        ));

        // Test Error
        assert!(matches!(quoted(br#"\"#), Err(nom::Err::Error(_))));
    }

    #[test]
    fn test_quoted_char() {
        let (rem, val) = quoted_char(b"\\\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, QuotedChar::try_from('"').unwrap());
    }

    #[test]
    fn test_number() {
        assert!(number(b"").is_err());
        assert!(number(b"?").is_err());

        assert!(number(b"0?").is_ok());
        assert!(number(b"55?").is_ok());
        assert!(number(b"999?").is_ok());
    }

    #[test]
    fn test_nz_number() {
        assert!(number(b"").is_err());
        assert!(number(b"?").is_err());

        assert!(nz_number(b"0?").is_err());
        assert!(nz_number(b"55?").is_ok());
        assert!(nz_number(b"999?").is_ok());
    }

    #[test]
    fn test_literal() {
        assert!(literal(b"{3}\r\n123").is_ok());
        assert!(literal(b"{3}\r\n1\x003").is_err());

        let (rem, val) = literal(b"{3}\r\n123xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, Literal::try_from(b"123".as_slice()).unwrap());
    }

    #[test]
    fn test_nil() {
        assert!(nil(b"nil").is_ok());
        assert!(nil(b"nil ").is_ok());
        assert!(nil(b" nil").is_err());
        assert!(nil(b"null").is_err());

        let (rem, _) = nil(b"nilxxx").unwrap();
        assert_eq!(rem, b"xxx");
    }
}
