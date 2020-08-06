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
    types::AuthMechanism,
};
use nom::{
    branch::alt,
    bytes::streaming::take_while1,
    combinator::{map, map_res},
    IResult,
};
use std::str::from_utf8;

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

// ----- Unsorted IMAP parsers -----

/// auth-type = atom ; Defined by [SASL]
pub fn auth_type(input: &[u8]) -> IResult<&[u8], AuthMechanism> {
    let (rem, raw_mechanism) = atom(input)?;

    // FIXME: just take inner String?
    let mechanism = match raw_mechanism.0.to_lowercase().as_ref() {
        "plain" => AuthMechanism::Plain,
        "login" => AuthMechanism::Login,
        _ => AuthMechanism::Other(raw_mechanism),
    };

    return Ok((rem, mechanism));
}

/// charset = atom / quoted
/// errata id: 261
pub fn charset(input: &[u8]) -> IResult<&[u8], String> {
    let parser = alt((
        map(atom, |val| val.0), // FIXME: just take String from Atom?
        map(quoted, |cow_str| cow_str.to_owned().to_string()), // TODO: is this correct?
    ));

    let (remaining, charset) = parser(input)?;

    Ok((remaining, charset))
}

/// tag = 1*<any ASTRING-CHAR except "+">
/// FIXME: this function has the _imap suffix to avoid confusion with
///        nom's "tag" parser. However, this function should be exposed
///        as "tag" to users of this library.
pub fn tag_imap(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take_while1(|b| is_astring_char(b) && b != b'+'), from_utf8)(input)
}
