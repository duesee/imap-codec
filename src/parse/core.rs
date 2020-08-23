use crate::{
    parse::mailbox::is_list_wildcards,
    types::core::{unescape_quoted, AString, Atom, NString, Nil, String as IMAPString},
};
use abnf_core::streaming::{is_CHAR, is_CTL, CRLF_relaxed as CRLF, DQUOTE};
use nom::{
    branch::alt,
    bytes::streaming::{escaped, tag, tag_no_case, take, take_while1, take_while_m_n},
    character::streaming::{digit1, one_of},
    combinator::{map, map_res, value},
    error::ErrorKind,
    sequence::{delimited, tuple},
    IResult,
};
use std::{borrow::Cow, str::from_utf8};

// ----- number -----

/// number = 1*DIGIT
///           ; Unsigned 32-bit integer
///           ; (0 <= n < 4,294,967,296)
pub fn number(input: &[u8]) -> IResult<&[u8], u32> {
    let parser = map_res(map_res(digit1, from_utf8), str::parse::<u32>);

    let (remaining, number) = parser(input)?;

    Ok((remaining, number))
}

/// nz-number = digit-nz *DIGIT
///              ; Non-zero unsigned 32-bit integer
///              ; (0 < n < 4,294,967,296)
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
    match byte {
        b'1'..=b'9' => true,
        _ => false,
    }
}

// ----- string -----

/// string = quoted / literal
pub fn string(input: &[u8]) -> IResult<&[u8], IMAPString> {
    let parser = alt((
        map(quoted, |cow_str| {
            IMAPString::Quoted(cow_str.to_owned().to_string())
        }), // TODO: is this correct?
        map(literal, |bytes| IMAPString::Literal(bytes.to_owned())),
    ));

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
            tuple((
                tag("\\"),
                take_while_m_n(1, 1, |byte| is_quoted_specials(byte)),
            )),
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
pub fn astring(input: &[u8]) -> IResult<&[u8], AString> {
    let parser = alt((
        map(take_while1(is_astring_char), |bytes: &[u8]| {
            AString::Atom(String::from_utf8(bytes.to_vec()).unwrap())
        }),
        map(string, AString::String),
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
pub fn atom(input: &[u8]) -> IResult<&[u8], Atom> {
    let parser = take_while1(is_atom_char);

    let (remaining, parsed_atom) = parser(input)?;

    Ok((
        remaining,
        Atom(String::from_utf8(parsed_atom.to_vec()).unwrap()),
    ))
}

// ----- nstring ----- nil or string

/// nstring = string / nil
pub fn nstring(input: &[u8]) -> IResult<&[u8], NString> {
    let parser = alt((map(string, NString::String), map(nil, |_| NString::Nil)));

    let (remaining, parsed_nstring) = parser(input)?;

    Ok((remaining, parsed_nstring))
}

/// nil = "NIL"
pub fn nil(input: &[u8]) -> IResult<&[u8], Nil> {
    let parser = value(Nil, tag_no_case(b"NIL"));

    let (remaining, parsed_nil) = parser(input)?;

    Ok((remaining, parsed_nil))
}

// ----- text -----

/// text = 1*TEXT-CHAR
pub fn text(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take_while1(is_text_char), from_utf8)(input)
}

/// TEXT-CHAR = %x01-09 / %x0B-0C / %x0E-7F
///               ; mod: was <any CHAR except CR and LF>
pub fn is_text_char(c: u8) -> bool {
    match c {
        0x01..=0x09 | 0x0b..=0x0c | 0x0e..=0x7f => true,
        _ => false,
    }
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
        assert_eq!(val, Atom("a".into()));
        assert_eq!(rem, b"(");

        let (rem, val) = atom(b"xxx yyy").unwrap();
        assert_eq!(val, Atom("xxx".into()));
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

        let (rem, val) = nil(b"nilxxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, Nil);
    }
}
