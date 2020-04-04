use crate::{
    parse::{crlf, dquote, is_digit, one},
    types::core::{AString, Atom, NString, Nil, String as IMAPString},
};
use nom::{
    branch::alt,
    bytes::streaming::{escaped, tag, tag_no_case, take, take_while1},
    character::streaming::digit1,
    combinator::{map, map_res, value},
    error::ErrorKind,
    sequence::tuple,
    IResult,
};
use std::str::from_utf8;

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
pub fn digit_nz(input: &[u8]) -> IResult<&[u8], u8> {
    let parser = one(|c| is_digit(c) && c != 0);

    let (remaining, parsed_digit_nz) = parser(input)?;

    Ok((remaining, parsed_digit_nz))
}

// ----- string -----

/// string = quoted / literal
pub fn string(input: &[u8]) -> IResult<&[u8], IMAPString> {
    let parser = alt((
        map(quoted, IMAPString::Quoted),
        map(literal, |bytes| IMAPString::Literal(bytes.to_vec())),
    ));

    let (remaining, parsed_string) = parser(input)?;

    Ok((remaining, parsed_string))
}

/// quoted = DQUOTE *QUOTED-CHAR DQUOTE
pub fn quoted(input: &[u8]) -> IResult<&[u8], String> {
    let parser = tuple((
        dquote,
        escaped(one(is_quoted_char_inner), '\\', quoted_specials),
        dquote,
    ));

    let (remaining, (_, quoted, _)) = parser(input)?;

    Ok((remaining, String::from_utf8(quoted.to_vec()).unwrap()))
}

pub fn is_quoted_char_inner(c: u8) -> bool {
    match c {
        0x01..=0x09 | 0x0b..=0x0c | 0x0e..=0x21 | 0x23..=0x5b | 0x5d..=0x7f => true,
        _ => false,
    }
}

/// QUOTED-CHAR = (%x01-09 / %x0B-0C / %x0E-21 / %x23-5B / %x5D-7F) / "\" quoted-specials
///                 ; mod: was <any TEXT-CHAR except quoted-specials> / "\" quoted-specials
pub fn quoted_char(input: &[u8]) -> IResult<&[u8], String> {
    let parser = alt((
        map(one(is_quoted_char_inner), |c| format!("{}", c)),
        map(tuple((tag(b"\\"), quoted_specials)), |(bs, qs)| {
            let mut val = Vec::new();
            val.extend_from_slice(bs);
            val.extend_from_slice(qs);
            String::from_utf8(val).unwrap()
        }),
    ));

    let (_remaining, _parsed_quoted_char) = parser(input)?;

    unimplemented!();
}

/// quoted-specials = DQUOTE / "\"
pub fn quoted_specials(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let parser = alt((dquote, tag(b"\\")));

    let (remaining, parsed_quoted_specials) = parser(input)?;

    Ok((remaining, parsed_quoted_specials))
}

/// literal = "{" number "}" CRLF *CHAR8
///             ; Number represents the number of CHAR8s
pub fn literal(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let parser = tuple((tag(b"{"), number, tag(b"}"), crlf));

    let (remaining, (_, number, _, _)) = parser(input)?;

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
            AString::Atom(Atom(String::from_utf8(bytes.to_vec()).unwrap()))
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

/// ATOM-CHAR = %x21 / %x23-24 / %x26-27 / %x2B-5B / %x5E-7A / %x7C-7E
///               ; mod: was <any CHAR except atom-specials>
///               ;
///               ; atom-specials = "(" / ")" / "{" / SP / CTL / list-wildcards / quoted-specials / resp-specials
pub fn is_atom_char(i: u8) -> bool {
    match i {
        0x21 | 0x23 | 0x24 | 0x26 | 0x27 | 0x2b..=0x5b | 0x5e..=0x7a | 0x7c..=0x7e => true,
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

#[cfg(test)]
mod test {
    use super::*;

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

    //#[test]
    //fn test_string() {
    //    unimplemented!();
    //}

    #[test]
    fn test_quoted() {
        let (rem, val) = quoted(b"\"asdasd\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, "asdasd".to_string());
    }

    //#[test]
    //fn test_quoted_char() {
    //    unimplemented!();
    //}

    //#[test]
    //fn test_quoted_specials() {
    //    unimplemented!();
    //}

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

    //#[test]
    //fn test_astring() {
    //    unimplemented!();
    //}

    #[test]
    //fn test_astring_char() {
    //    unimplemented!();
    //}
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
