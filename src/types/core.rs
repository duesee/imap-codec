//! # 4. Data Formats
//!
//! IMAP4rev1 uses textual commands and responses.  Data in
//! IMAP4rev1 can be in one of several forms: atom, number, string,
//! parenthesized list, or NIL.  Note that a particular data item
//! may take more than one form; for example, a data item defined as
//! using "astring" syntax may be either an atom or a string.

use crate::{
    codec::Encode,
    parse::core::{is_astring_char, is_atom_char, is_text_char},
    utils::escape_quoted,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, convert::TryFrom, fmt, io::Write, string::FromUtf8Error};

// ## 4.1. Atom

/// An atom consists of one or more non-special characters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Atom(String);

impl TryFrom<&str> for Atom {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Atom::try_from(value.to_string())
    }
}

impl TryFrom<String> for Atom {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO: use `atom` parser directly?
        if value.is_empty() {
            Err(())
        } else if value.bytes().all(is_atom_char) {
            Ok(Atom(value))
        } else {
            Err(())
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Encode for Atom {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum IString {
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
    /// FIXME: not every String (UTF-8) is a valid "quoted IMAP string"
    Quoted(String),
}

impl From<&str> for IString {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for IString {
    fn from(s: String) -> Self {
        if s.chars().all(|c| c.is_ascii() && is_text_char(c as u8)) {
            IString::Quoted(s)
        } else {
            IString::Literal(s.into_bytes()) // FIXME: \x00 not allowed, but may be present in UTF8-String
        }
    }
}

impl TryFrom<IString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: IString) -> Result<Self, Self::Error> {
        match value {
            IString::Quoted(utf8) => Ok(utf8),
            IString::Literal(bytes) => String::from_utf8(bytes), // FIXME(misuse): must not contain \x00
        }
    }
}

impl Encode for IString {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Literal(val) => {
                write!(writer, "{{{}}}\r\n", val.len())?;
                writer.write_all(val)
            }
            Self::Quoted(val) => write!(writer, "\"{}\"", escape_quoted(val)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NString(pub Option<IString>);

impl Encode for NString {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self.0 {
            Some(imap_str) => imap_str.encode(writer),
            None => writer.write_all(b"NIL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AString {
    Atom(String),
    String(IString),
}

impl From<&str> for AString {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for AString {
    fn from(s: String) -> Self {
        if s.is_empty() {
            AString::String("".into())
        } else if s.chars().all(|c| c.is_ascii() && is_astring_char(c as u8)) {
            AString::Atom(s)
        } else {
            AString::String(s.into())
        }
    }
}

impl TryFrom<AString> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: AString) -> Result<Self, Self::Error> {
        match value {
            AString::Atom(string) => Ok(string),
            AString::String(istring) => String::try_from(istring),
        }
    }
}

impl Encode for AString {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            AString::Atom(atom) => writer.write_all(atom.as_bytes()),
            AString::String(imap_str) => imap_str.encode(writer),
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Tag(pub(crate) String);

impl TryFrom<&str> for Tag {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Tag::try_from(value.to_string())
    }
}

impl TryFrom<String> for Tag {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.bytes().all(|c| is_astring_char(c) && c != b'+') {
            Ok(Tag(value))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Encode for Tag {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct Text(pub(crate) String);

impl TryFrom<&str> for Text {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Text::try_from(value.to_string())
    }
}

impl TryFrom<String> for Text {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err("Text must not be empty.")
        } else {
            if value.bytes().all(is_text_char) {
                Ok(Text(value.to_string()))
            } else {
                Err("Text contains illegal characters.")
            }
        }
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Encode for Text {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Charset(pub(crate) String);

impl TryFrom<&str> for Charset {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Charset::try_from(value.to_string())
    }
}

impl TryFrom<String> for Charset {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.chars().all(|c| c.is_ascii() && is_text_char(c as u8)) {
            Ok(Charset(value))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "\"\"")
        } else if self // TODO: don't check every time...
            .0
            .chars()
            .all(|c| c.is_ascii() && is_atom_char(c as u8))
        {
            write!(f, "{}", self.0)
        } else {
            write!(f, "\"{}\"", &escape_quoted(&self.0))
        }
    }
}

impl Encode for Charset {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        // FIXME(perf): conversion calls should not
        //              be requires for serialization.
        writer.write_all(self.to_string().as_bytes())
    }
}

// ----- "Referenced types" used for non-allocating code -----

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct atm<'a>(pub(crate) &'a str);

impl<'a> atm<'a> {
    pub fn to_owned(&self) -> Atom {
        Atom(self.0.to_string())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum istr<'a> {
    Literal(&'a [u8]),
    Quoted(Cow<'a, str>),
}

impl<'a> istr<'a> {
    pub fn to_owned(&self) -> IString {
        match self {
            istr::Literal(bytes) => IString::Literal(bytes.to_vec()),
            istr::Quoted(cowstr) => IString::Quoted(cowstr.to_string()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct nstr<'a>(pub Option<istr<'a>>);

impl<'a> nstr<'a> {
    pub fn to_owned(&self) -> NString {
        NString(self.0.as_ref().map(|inner| inner.to_owned()))
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum astr<'a> {
    Atom(&'a str),
    String(istr<'a>),
}

impl<'a> astr<'a> {
    pub fn to_owned(&self) -> AString {
        match self {
            astr::Atom(str) => AString::Atom(str.to_string()),
            astr::String(istr) => AString::String(istr.to_owned()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct txt<'a>(pub(crate) &'a str);

impl<'a> txt<'a> {
    pub fn to_owned(&self) -> Text {
        Text(self.0.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_conversion() {
        assert_eq!(IString::from("AAA"), IString::Quoted("AAA".into()).into());
        assert_eq!(
            IString::from("\"AAA"),
            IString::Quoted("\"AAA".into()).into()
        );

        assert_ne!(
            IString::from("\"AAA"),
            IString::Quoted("\\\"AAA".into()).into()
        );
    }

    #[test]
    fn test_charset() {
        let tests = [
            ("bengali", "bengali"),
            ("\"simple\" english", r#""\"simple\" english""#),
            ("", "\"\""),
            ("\"", "\"\\\"\""),
            ("\\", "\"\\\\\""),
        ];

        for (from, expected) in tests.iter() {
            let cs = Charset::try_from(*from).unwrap();
            println!("{}", cs);

            let mut out = Vec::new();
            cs.encode(&mut out).unwrap();
            assert_eq!(String::from_utf8(out).unwrap(), *expected);
        }

        assert!(Charset::try_from("\r").is_err());
        assert!(Charset::try_from("\n").is_err());
        assert!(Charset::try_from("¹").is_err());
        assert!(Charset::try_from("²").is_err());
        assert!(Charset::try_from("\x00").is_err());
    }
}
