use crate::{
    parse::mailbox::is_list_wildcards,
    types::core::{astr, atm, istr, nstr, unescape_quoted, Charset, Tag},
};
use abnf_core::streaming::{is_ALPHA, is_CHAR, is_CTL, is_DIGIT, CRLF_relaxed as CRLF, DQUOTE};
use nom::{
    branch::alt,
    bytes::streaming::{escaped, tag, tag_no_case, take, take_while, take_while1, take_while_m_n},
    character::streaming::{digit1, one_of},
    combinator::{map, map_res, opt, recognize, value},
    error::ErrorKind,
    sequence::{delimited, tuple},
    IResult,
};
use std::{borrow::Cow, str::from_utf8};

// ----- number -----

/// Unsigned 32-bit integer (0 <= n < 4,294,967,296)
///
/// number = 1*DIGIT
pub fn number(input: &[u8]) -> IResult<&[u8], u32> {
    let parser = map_res(map_res(digit1, from_utf8), str::parse::<u32>);

    let (remaining, number) = parser(input)?;

    Ok((remaining, number))
}

/// Non-zero unsigned 32-bit integer (0 < n < 4,294,967,296)
///
/// nz-number = digit-nz *DIGIT
pub fn nz_number(input: &[u8]) -> IResult<&[u8], u32> {
    let (remaining, number) = number(input)?;

    if number == 0 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    Ok((remaining, number))
}

/// digit-nz = %x31-39 ; 1-9
pub fn is_digit_nz(byte: u8) -> bool {
    matches!(byte, b'1'..=b'9')
}

// ----- string -----

/// string = quoted / literal
pub fn string(input: &[u8]) -> IResult<&[u8], istr> {
    let parser = alt((map(quoted, istr::Quoted), map(literal, istr::Literal)));

    let (remaining, parsed_string) = parser(input)?;

    Ok((remaining, parsed_string))
}

/// quoted = DQUOTE *QUOTED-CHAR DQUOTE
///
/// This function only allocates a new String, when needed, i.e. when
/// quoted chars need to be replaced.
pub fn quoted(input: &[u8]) -> IResult<&[u8], Cow<str>> {
    let parser = tuple((
        DQUOTE,
        map_res(
            escaped(
                take_while1(is_any_text_char_except_quoted_specials),
                '\\',
                one_of("\\\""),
            ),
            from_utf8,
        ),
        DQUOTE,
    ));

    let (remaining, (_, quoted, _)) = parser(input)?;

    Ok((remaining, unescape_quoted(quoted)))
}

/// QUOTED-CHAR = <any TEXT-CHAR except quoted-specials> / "\" quoted-specials
pub fn quoted_char(input: &[u8]) -> IResult<&[u8], char> {
    let parser = alt((
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
    ));

    let (remaining, quoted_char) = parser(input)?;

    Ok((remaining, quoted_char))
}

fn is_any_text_char_except_quoted_specials(byte: u8) -> bool {
    is_text_char(byte) && !is_quoted_specials(byte)
}

/// quoted-specials = DQUOTE / "\"
pub fn is_quoted_specials(byte: u8) -> bool {
    byte == b'"' || byte == b'\\'
}

/// literal = "{" number "}" CRLF *CHAR8
///             ; Number represents the number of CHAR8s
pub fn literal(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let parser = tuple((delimited(tag(b"{"), number, tag(b"}")), CRLF));

    let (remaining, (number, _)) = parser(input)?;

    let (remaining, data) = take(number)(remaining)?;

    // FIXME: what should we do?
    if !data.iter().cloned().all(is_char8) {
        return Err(nom::Err::Error((remaining, ErrorKind::Verify))); // FIXME: what ErrorKind should this have?
    }

    Ok((remaining, data))
}

/// CHAR8 = %x01-ff ; any OCTET except NUL, %x00
fn is_char8(i: u8) -> bool {
    i != 0
}

// ----- astring ----- atom (roughly) or string

/// astring = 1*ASTRING-CHAR / string
pub fn astring(input: &[u8]) -> IResult<&[u8], astr> {
    let parser = alt((
        map(take_while1(is_astring_char), |bytes: &[u8]| {
            astr::Atom(std::str::from_utf8(bytes).unwrap())
        }),
        map(string, astr::String),
    ));

    let (remaining, parsed_astring) = parser(input)?;

    Ok((remaining, parsed_astring))
}

/// ASTRING-CHAR = ATOM-CHAR / resp-specials
pub fn is_astring_char(i: u8) -> bool {
    is_atom_char(i) || is_resp_specials(i)
}

/// ATOM-CHAR = <any CHAR except atom-specials>
pub fn is_atom_char(b: u8) -> bool {
    is_CHAR(b) && !is_atom_specials(b)
}

/// atom-specials = "(" / ")" / "{" / SP / CTL / list-wildcards / quoted-specials / resp-specials
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

/// resp-specials = "]"
pub fn is_resp_specials(i: u8) -> bool {
    i == b']'
}

/// atom = 1*ATOM-CHAR
pub fn atom(input: &[u8]) -> IResult<&[u8], atm> {
    let parser = take_while1(is_atom_char);

    let (remaining, parsed_atom) = parser(input)?;

    Ok((remaining, atm(std::str::from_utf8(parsed_atom).unwrap())))
}

// ----- nstring ----- nil or string

/// nstring = string / nil
pub fn nstring(input: &[u8]) -> IResult<&[u8], nstr> {
    let parser = alt((
        map(string, |item| nstr(Some(item))),
        map(nil, |_| nstr(None)),
    ));

    let (remaining, parsed_nstring) = parser(input)?;

    Ok((remaining, parsed_nstring))
}

/// nil = "NIL"
pub fn nil(input: &[u8]) -> IResult<&[u8], ()> {
    value((), tag_no_case(b"NIL"))(input)
}

// ----- text -----

/// text = 1*TEXT-CHAR
pub fn text(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take_while1(is_text_char), from_utf8)(input)
}

/// TEXT-CHAR = %x01-09 / %x0B-0C / %x0E-7F
///               ; mod: was <any CHAR except CR and LF>
pub fn is_text_char(c: u8) -> bool {
    matches!(c, 0x01..=0x09 | 0x0b..=0x0c | 0x0e..=0x7f)
}

// ----- base64 -----

/// base64 = *(4base64-char) [base64-terminal]
pub fn base64(input: &[u8]) -> IResult<&[u8], &str> {
    let parser = map_res(
        recognize(tuple((
            take_while(is_base64_char),
            opt(alt((tag("=="), tag("=")))),
        ))),
        from_utf8,
    );

    let (remaining, base64) = parser(input)?;

    Ok((remaining, base64))
}

/// base64-char = ALPHA / DIGIT / "+" / "/" ; Case-sensitive
fn is_base64_char(i: u8) -> bool {
    is_ALPHA(i) || is_DIGIT(i) || i == b'+' || i == b'/'
}

// base64-terminal = (2base64-char "==") / (3base64-char "=")

// ----- charset -----

/// charset = atom / quoted
/// errata id: 261
pub fn charset(input: &[u8]) -> IResult<&[u8], Charset> {
    let parser = alt((
        map(atom, |val| Charset(val.0.to_string())),
        map(quoted, |cow| Charset(cow.to_string())),
    ));

    let (remaining, charset) = parser(input)?;

    Ok((remaining, charset))
}

// ----- tag -----

/// tag = 1*<any ASTRING-CHAR except "+">
pub(crate) fn tag_imap(input: &[u8]) -> IResult<&[u8], Tag> {
    map(
        map_res(take_while1(|b| is_astring_char(b) && b != b'+'), from_utf8),
        |s| Tag(s.to_string()),
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn test_atom() {
        assert!(atom(b" ").is_err());
        assert!(atom(b"").is_err());

        let (rem, val) = atom(b"a(").unwrap();
        assert_eq!(val, atm("a"));
        assert_eq!(rem, b"(");

        let (rem, val) = atom(b"xxx yyy").unwrap();
        assert_eq!(val, atm("xxx"));
        assert_eq!(rem, b" yyy");
    }

    #[test]
    fn test_quoted() {
        let (rem, val) = quoted(br#""Hello"???"#).unwrap();
        assert_eq!(rem, b"???");
        assert_eq!(val, "Hello");

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
        assert_eq!(val, r#"Hello "World""#); // fails

        // Test Incomplete
        assert_matches!(quoted(br#""#), Err(nom::Err::Incomplete(_)));
        assert_matches!(quoted(br#""\"#), Err(nom::Err::Incomplete(_)));
        assert_matches!(quoted(br#""Hello "#), Err(nom::Err::Incomplete(_)));

        // Test Error
        assert_matches!(quoted(br#"\"#), Err(nom::Err::Error(_)));
    }

    #[test]
    fn test_quoted_char() {
        let (rem, val) = quoted_char(b"\\\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, '"');
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
        assert_eq!(val, b"123");
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
