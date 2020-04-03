//! # 9. Formal Syntax
//!
//! The following syntax specification uses the Augmented Backus-Naur
//! Form (ABNF) notation as specified in [ABNF].
//!
//! In the case of alternative or optional rules in which a later rule
//! overlaps an earlier rule, the rule which is listed earlier MUST take
//! priority.  For example, "\Seen" when parsed as a flag is the \Seen
//! flag name and not a flag-extension, even though "\Seen" can be parsed
//! as a flag-extension.  Some, but not all, instances of this rule are
//! noted below.
//!
//! ### Note
//!
//! [ABNF] rules MUST be followed strictly; in particular:
//!
//! * (1) Except as noted otherwise, all alphabetic characters
//! are case-insensitive.  The use of upper or lower case
//! characters to define token strings is for editorial clarity
//! only.  Implementations MUST accept these strings in a
//! case-insensitive fashion.
//! * (2) In all cases, SP refers to exactly one space.  It is
//! NOT permitted to substitute TAB, insert additional spaces,
//! or otherwise treat SP as being equivalent to LWSP.
//! * (3) The ASCII NUL character, %x00, MUST NOT be used at any
//! time.

use crate::{
    parse::core::{atom, is_astring_char, quoted},
    types::core::Atom,
};
use nom::{
    branch::alt, bytes::streaming::tag as nom_tag, bytes::streaming::take_while1,
    character::streaming::line_ending, combinator::map, error::ParseError, Err, Err::Incomplete,
    IResult, Needed,
};

pub mod address;
pub mod base64;
pub mod body;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
pub mod flag;
pub mod header;
pub mod mailbox;
pub mod message;
pub mod response;
pub mod section;
pub mod sequence;
pub mod status;

pub fn one<'a, F, Error: ParseError<&'a [u8]>>(
    cond: F,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], u8, Error>
where
    F: Fn(u8) -> bool,
{
    move |i| {
        if i.is_empty() {
            Err(Incomplete(Needed::Size(1)))
        } else if cond(i[0]) {
            Ok((&i[1..], i[0]))
        } else {
            Err(Err::Error(Error::from_char(i, i[0] as char)))
        }
    }
}

pub fn _range<'a, Input: 'a, Error: nom::error::ParseError<Input>>(
    _min: u8,
    _max: u8,
) -> impl Fn(Input) -> IResult<Input, Input, Error> {
    move |_i: Input| unimplemented!()
}

// ----- Required ABNF Core Rules (RFC5234 B.1.) -----

/// ALPHA = %x41-5A / %x61-7A ; A-Z / a-z
pub fn is_alpha(i: u8) -> bool {
    match i as char {
        'a'..='z' | 'A'..='Z' => true,
        _ => false,
    }
}

/// CRLF = CR LF ; Internet standard newline
/// TODO: Also accepts "\n" only (without "\r".)
pub fn crlf(input: &[u8]) -> IResult<&[u8], &[u8]> {
    line_ending(input)
}

pub fn is_digit(i: u8) -> bool {
    match i {
        b'0'..=b'9' => true,
        _ => false,
    }
}

/// DIGIT = %x30-39 ; 0-9
/// FIXME: this function returns u8 as ascii
pub fn digit(input: &[u8]) -> IResult<&[u8], u8> {
    one(is_digit)(input)
}

/// DQUOTE = %x22 ; " (Double Quote)
pub fn dquote(input: &[u8]) -> IResult<&[u8], &[u8]> {
    nom_tag("\"")(input)
}

/// SP = %x20
pub fn sp(input: &[u8]) -> IResult<&[u8], &[u8]> {
    nom_tag(" ")(input)
}

// ----- Unsorted IMAP parsers -----

/// auth-type = atom ; Defined by [SASL]
pub fn auth_type(input: &[u8]) -> IResult<&[u8], Atom> {
    atom(input)
}

/// charset = atom / quoted
/// errata id: 261
pub fn charset(input: &[u8]) -> IResult<&[u8], String> {
    let parser = alt((
        map(atom, |val| val.0), // TODO: really make a std::string::String out of newtype Atom?
        quoted,                 // TODO: quoted is already a std::string::String
    ));

    let (remaining, charset) = parser(input)?;

    Ok((remaining, charset))
}

/// tag = 1*<any ASTRING-CHAR except "+">
pub fn tag(input: &[u8]) -> IResult<&[u8], String> {
    let parser = take_while1(|b| is_astring_char(b) && b != b'+');

    let (remaining, parsed_tag_) = parser(input)?;

    Ok((remaining, String::from_utf8(parsed_tag_.to_vec()).unwrap()))
}
