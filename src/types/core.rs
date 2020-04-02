//! # 4. Data Formats
//!
//! IMAP4rev1 uses textual commands and responses.  Data in
//! IMAP4rev1 can be in one of several forms: atom, number, string,
//! parenthesized list, or NIL.  Note that a particular data item
//! may take more than one form; for example, a data item defined as
//! using "astring" syntax may be either an atom or a string.

use crate::{
    codec::{escape, Codec},
    parse::core::is_quoted_char_inner,
};
use serde::Deserialize;
use std::{convert::TryFrom, fmt};

// ## 4.1. Atom
//
// An atom consists of one or more non-special characters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Atom(pub std::string::String);

impl Codec for Atom {
    fn serialize(&self) -> Vec<u8> {
        self.0.as_bytes().to_owned()
    }

    fn deserialize(input: &[u8]) -> Result<(&[u8], Self), std::string::String>
    where
        Self: Sized,
    {
        use crate::parse::core::atom;

        atom(input).map_err(|_| "Error parsing Atom".to_string())
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

// ## 4.2. Number
//
// A number consists of one or more digit characters, and
// represents a numeric value.

pub type Number = u32;

// ## 4.3. String

/// A string is in one of two forms: either literal or quoted string.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum String {
    /// The literal form is the general form of string.
    /// FIXME: must not contain a zero (\x00)
    Literal(Vec<u8>),
    /// The quoted string form is an alternative that avoids the overhead of
    /// processing a literal at the cost of limitations of characters
    /// which may be used.
    /// FIXME: not every std::string::String (UTF-8) is a valid "quoted IMAP string"
    Quoted(std::string::String),
}

impl Codec for String {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::Quoted(val) => [b"\"", val.as_bytes(), b"\""].concat(),
            Self::Literal(val) => [
                b"{",
                val.len().to_string().as_bytes(),
                b"}",
                b"\r\n",
                val.as_ref(),
            ]
            .concat(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), std::string::String>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl std::fmt::Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Quoted(val) => write!(f, "\"{}\"", val),
            Self::Literal(val) => write!(
                f,
                "{{{}}}\r\n{}",
                val.len(),
                std::string::String::from_utf8(val.to_owned())
                    .expect("not every literal is valid UTF-8...")
            ),
        }
    }
}

impl TryFrom<std::string::String> for String {
    type Error = std::string::String;

    fn try_from(value: std::string::String) -> Result<Self, Self::Error> {
        if value.is_ascii() && value.bytes().all(is_quoted_char_inner) {
            Ok(String::Quoted(value))
        } else {
            Err(format!(
                "String \"{}\" contains data, which is invalid for an imap string",
                escape(value.as_bytes())
            ))
        }
    }
}

impl TryFrom<&'static str> for String {
    type Error = &'static str;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        if value.is_ascii() && value.bytes().all(is_quoted_char_inner) {
            Ok(String::Quoted(value.to_owned()))
        } else {
            Err("String contains data, which is invalid for an imap string")
        }
    }
}

// A literal is a sequence of zero or more octets (including CR and
// LF), prefix-quoted with an octet count in the form of an open
// brace ("{"), the number of octets, close brace ("}"), and CRLF.
// In the case of literals transmitted from server to client, the
// CRLF is immediately followed by the octet data.  In the case of
// literals transmitted from client to server, the client MUST wait
// to receive a command continuation request (described later in
// this document) before sending the octet data (and the remainder
// of the command).
//
// A quoted string is a sequence of zero or more 7-bit characters,
// excluding CR and LF, with double quote (<">) characters at each
// end.
//
// The empty string is represented as either "" (a quoted string
// with zero characters between double quotes) or as {0} followed
// by CRLF (a literal with an octet count of 0).
//
//   Note: Even if the octet count is 0, a client transmitting a
//   literal MUST wait to receive a command continuation request.

// TODO: use `Option<String>` instead?
#[derive(Debug, Clone, PartialEq)]
pub enum NString {
    Nil,
    String(String),
}

impl std::fmt::Display for NString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::String(imap_str) => write!(f, "{}", imap_str),
        }
    }
}

impl From<std::string::String> for NString {
    fn from(val: std::string::String) -> Self {
        NString::String(String::try_from(val).unwrap())
    }
}

impl From<&'static str> for NString {
    fn from(val: &'static str) -> Self {
        NString::String(String::try_from(val).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum AString {
    Atom(Atom),
    String(String),
}

impl Codec for AString {
    fn serialize(&self) -> Vec<u8> {
        match self {
            AString::Atom(atom) => atom.serialize(),
            AString::String(imap_str) => imap_str.serialize(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), std::string::String>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl fmt::Display for AString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            AString::Atom(atom) => write!(f, "{}", atom),
            AString::String(_imap_str) => write!(f, "{}", _imap_str),
        }
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
