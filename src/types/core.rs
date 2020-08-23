//! # 4. Data Formats
//!
//! IMAP4rev1 uses textual commands and responses.  Data in
//! IMAP4rev1 can be in one of several forms: atom, number, string,
//! parenthesized list, or NIL.  Note that a particular data item
//! may take more than one form; for example, a data item defined as
//! using "astring" syntax may be either an atom or a string.

use crate::codec::Codec;
use serde::Deserialize;
use std::{borrow::Cow, fmt};

// ## 4.1. Atom

/// An atom consists of one or more non-special characters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Atom(pub std::string::String);

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Codec for Atom {
    fn serialize(&self) -> Vec<u8> {
        format!("{}", self.0).into_bytes()
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Atom>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

// ## 4.2. Number
//
// A number consists of one or more digit characters, and
// represents a numeric value.

pub type Number = u32;

// ## 4.3. String

/// A string is in one of two forms: either literal or quoted string.
///
/// The empty string is represented as either "" (a quoted string
/// with zero characters between double quotes) or as {0} followed
/// by CRLF (a literal with an octet count of 0).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum String {
    /// A literal is a sequence of zero or more octets (including CR and
    /// LF), prefix-quoted with an octet count in the form of an open
    /// brace ("{"), the number of octets, close brace ("}"), and CRLF.
    /// In the case of literals transmitted from server to client, the
    /// CRLF is immediately followed by the octet data.  In the case of
    /// literals transmitted from client to server, the client MUST wait
    /// to receive a command continuation request (...) before sending
    /// the octet data (and the remainder of the command).
    ///
    /// Note: Even if the octet count is 0, a client transmitting a
    /// literal MUST wait to receive a command continuation request.
    ///
    /// FIXME: must not contain a zero (\x00)
    Literal(Vec<u8>),
    /// The quoted string form is an alternative that avoids the overhead of
    /// processing a literal at the cost of limitations of characters which may be used.
    ///
    /// A quoted string is a sequence of zero or more 7-bit characters,
    /// excluding CR and LF, with double quote (<">) characters at each end.
    ///
    /// FIXME: not every std::string::String (UTF-8) is a valid "quoted IMAP string"
    Quoted(std::string::String),
}

pub fn escape_quoted<'a>(unescaped: &'a str) -> Cow<'a, str> {
    let mut escaped = Cow::Borrowed(unescaped);

    if escaped.contains("\\") {
        escaped = Cow::Owned(escaped.replace("\\", "\\\\"));
    }

    if escaped.contains("\"") {
        escaped = Cow::Owned(escaped.replace("\"", "\\\""));
    }

    escaped
}

pub fn unescape_quoted<'a>(escaped: &'a str) -> Cow<'a, str> {
    let mut unescaped = Cow::Borrowed(escaped);

    if unescaped.contains("\\\\") {
        unescaped = Cow::Owned(unescaped.replace("\\\\", "\\"));
    }

    if unescaped.contains("\\\"") {
        unescaped = Cow::Owned(unescaped.replace("\\\"", "\""));
    }

    unescaped
}

impl Codec for String {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::Literal(val) => {
                let mut out = format!("{{{}}}\r\n", val.len()).into_bytes();
                out.extend_from_slice(val);
                out
            }
            Self::Quoted(val) => format!("\"{}\"", escape_quoted(val)).into_bytes(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), String>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

// TODO: use `Option<String>` instead?
#[derive(Debug, Clone, PartialEq)]
pub enum NString {
    Nil,
    String(String),
}

impl Codec for NString {
    fn serialize(&self) -> Vec<u8> {
        match self {
            NString::Nil => b"NIL".to_vec(),
            NString::String(imap_str) => imap_str.serialize(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), NString>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl From<std::string::String> for NString {
    fn from(val: std::string::String) -> Self {
        NString::String(String::Quoted(val))
    }
}

impl From<&'static str> for NString {
    fn from(val: &'static str) -> Self {
        NString::String(String::Quoted(val.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum AString {
    Atom(std::string::String),
    String(String),
}

impl Codec for AString {
    fn serialize(&self) -> Vec<u8> {
        match self {
            AString::Atom(atom) => atom.as_bytes().to_vec(),
            AString::String(imap_str) => imap_str.serialize(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), AString>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

// 4.3.1.  8-bit and Binary Strings
//
//    8-bit textual and binary mail is supported through the use of a
//    [MIME-IMB] content transfer encoding.  IMAP4rev1 implementations MAY
//    transmit 8-bit or multi-octet characters in literals, but SHOULD do
//    so only when the [CHARSET] is identified.
//
//    Although a BINARY body encoding is defined, unencoded binary strings
//    are not permitted.  A "binary string" is any string with NUL
//    characters.  Implementations MUST encode binary data into a textual
//    form, such as BASE64, before transmitting the data.  A string with an
//    excessive amount of CTL characters MAY also be considered to be
//    binary.

// 4.4.    Parenthesized List
//
//    Data structures are represented as a "parenthesized list"; a sequence
//    of data items, delimited by space, and bounded at each end by
//    parentheses.  A parenthesized list can contain other parenthesized
//    lists, using multiple levels of parentheses to indicate nesting.
//
//    The empty list is represented as () -- a parenthesized list with no
//    members.

/// 4.5. NIL
///
/// The special form "NIL" represents the non-existence of a particular
/// data item that is represented as a string or parenthesized list, as
/// distinct from the empty string "" or the empty parenthesized list ().
///
///  Note: NIL is never used for any data item which takes the
///  form of an atom.  For example, a mailbox name of "NIL" is a
///  mailbox named NIL as opposed to a non-existent mailbox
///  name.  This is because mailbox uses "astring" syntax which
///  is an atom or a string.  Conversely, an addr-name of NIL is
///  a non-existent personal name, because addr-name uses
///  "nstring" syntax which is NIL or a string, but never an
///  atom.
#[derive(Debug, Clone, PartialEq)]
pub struct Nil;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_escape_quoted() {
        assert_eq!(escape_quoted("alice"), "alice");
        assert_eq!(escape_quoted("\\alice\\"), "\\\\alice\\\\");
        assert_eq!(escape_quoted("alice\""), "alice\\\"");
        assert_eq!(escape_quoted(r#"\alice\ ""#), r#"\\alice\\ \""#);
    }

    #[test]
    fn test_unescape_quoted() {
        assert_eq!(unescape_quoted("alice"), "alice");
        assert_eq!(unescape_quoted("\\\\alice\\\\"), "\\alice\\");
        assert_eq!(unescape_quoted("alice\\\""), "alice\"");
        assert_eq!(unescape_quoted(r#"\\alice\\ \""#), r#"\alice\ ""#);
    }
}
