//! # 4. Data Formats
//!
//! IMAP4rev1 uses textual commands and responses.  Data in
//! IMAP4rev1 can be in one of several forms: atom, number, string,
//! parenthesized list, or NIL.  Note that a particular data item
//! may take more than one form; for example, a data item defined as
//! using "astring" syntax may be either an atom or a string.

use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    fmt,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
    string::FromUtf8Error,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::{
    parse::core::{is_astring_char, is_atom_char, is_text_char},
    utils::escape_quoted,
};

// ## 4.1. Atom

/// An atom consists of one or more non-special characters.
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Atom(pub(crate) String);

impl TryFrom<&str> for Atom {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Atom::try_from(value.to_string())
    }
}

impl TryFrom<String> for Atom {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_empty() && value.bytes().all(is_atom_char) {
            Ok(Atom(value))
        } else {
            Err(())
        }
    }
}

impl Deref for Atom {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

// ## 4.2. Number
//
// A number consists of one or more digit characters, and
// represents a numeric value.

// ## 4.3. String

/// A string is in one of two forms: either literal or quoted string.
///
/// The empty string is represented as either "" (a quoted string
/// with zero characters between double quotes) or as {0} followed
/// by CRLF (a literal with an octet count of 0).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Literal(Literal),
    /// The quoted string form is an alternative that avoids the overhead of
    /// processing a literal at the cost of limitations of characters which may be used.
    ///
    /// A quoted string is a sequence of zero or more 7-bit characters,
    /// excluding CR and LF, with double quote (<">) characters at each end.
    ///
    Quoted(Quoted),
}

impl TryFrom<&str> for IString {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, ()> {
        s.to_string().try_into()
    }
}

impl TryFrom<String> for IString {
    type Error = ();

    fn try_from(s: String) -> Result<Self, ()> {
        if s.chars().all(|c| c.is_ascii() && is_text_char(c as u8)) {
            Ok(IString::Quoted(Quoted(s)))
        } else {
            let bytes = s.into_bytes();

            if bytes.iter().all(|b| *b != 0x00) {
                Ok(IString::Literal(Literal(bytes)))
            } else {
                Err(())
            }
        }
    }
}

impl TryFrom<IString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: IString) -> Result<Self, Self::Error> {
        match value {
            IString::Quoted(utf8) => Ok(utf8.0),
            IString::Literal(bytes) => String::from_utf8(bytes.0),
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal(Vec<u8>);

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralRef<'a>(&'a [u8]);

impl<'a> LiteralRef<'a> {
    pub fn verify(bytes: &[u8]) -> bool {
        bytes.iter().all(|byte| *byte != 0)
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Result<LiteralRef<'a>, ()> {
        if Self::verify(bytes) {
            Ok(Self(bytes))
        } else {
            Err(())
        }
    }

    pub unsafe fn from_bytes_unchecked(bytes: &'a [u8]) -> LiteralRef<'a> {
        Self(bytes)
    }
}

// Literal --> LiteralRef

impl<'a> From<&'a Literal> for LiteralRef<'a> {
    fn from(value: &'a Literal) -> LiteralRef<'a> {
        LiteralRef(&value.0)
    }
}

// LiteralRef --> Literal

impl<'a> From<&LiteralRef<'a>> for Literal {
    fn from(value: &LiteralRef<'a>) -> Literal {
        Literal(value.0.to_owned())
    }
}

impl<'a> TryFrom<&'a [u8]> for LiteralRef<'a> {
    type Error = ();

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        LiteralRef::from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for Literal {
    type Error = ();

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if LiteralRef::verify(&bytes) {
            Ok(Literal(bytes))
        } else {
            Err(())
        }
    }
}

impl Deref for Literal {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Deref for LiteralRef<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quoted(pub(crate) String);

impl TryFrom<&str> for Quoted {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TryFrom<String> for Quoted {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.chars().all(|c| c.is_ascii() && is_text_char(c as u8)) {
            Ok(Quoted(value))
        } else {
            Err(())
        }
    }
}

impl Deref for Quoted {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Quoted {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\"{}\"", escape_quoted(&self.0))
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NString(pub Option<IString>);

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AString {
    Atom(Atom),
    String(IString),
}

impl TryFrom<&str> for AString {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, ()> {
        s.to_string().try_into()
    }
}

impl TryFrom<String> for AString {
    type Error = ();

    fn try_from(s: String) -> Result<Self, ()> {
        if let Ok(atom) = Atom::try_from(s.clone()) {
            Ok(AString::Atom(atom))
        } else if let Ok(string) = IString::try_from(s) {
            Ok(AString::String(string))
        } else {
            Err(())
        }
    }
}

impl TryFrom<AString> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: AString) -> Result<Self, Self::Error> {
        match value {
            AString::Atom(string) => Ok(string.0),
            AString::String(istring) => String::try_from(istring),
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

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Tag(pub(crate) String);

impl Tag {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let buffer = [0u8; 8].map(|_| rng.sample(Alphanumeric));

        Self(unsafe { String::from_utf8_unchecked(buffer.to_vec()) })
    }

    pub fn verify(value: &str) -> bool {
        !value.is_empty() && value.bytes().all(|c| is_astring_char(c) && c != b'+')
    }
}

impl TryFrom<&str> for Tag {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Tag::try_from(value.to_string())
    }
}

impl TryFrom<String> for Tag {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Tag::verify(&value) {
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

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
        } else if value.bytes().all(is_text_char) {
            Ok(Text(value))
        } else {
            Err("Text contains illegal characters.")
        }
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Charset {
    Atom(Atom),
    Quoted(Quoted),
}

impl TryFrom<&str> for Charset {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Charset::try_from(value.to_string())
    }
}

impl TryFrom<String> for Charset {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Try Atom variant ...
        if let Ok(atom) = Atom::try_from(value.clone()) {
            // TODO(perf)
            Ok(Charset::Atom(atom))
        } else if let Ok(quoted) = Quoted::try_from(value) {
            Ok(Charset::Quoted(quoted))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Charset::Atom(atom) => write!(f, "{}", atom),
            Charset::Quoted(quoted) => write!(f, "{}", quoted),
        }
    }
}

// ----- "Referenced types" used for non-allocating code -----

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AtomRef<'a>(&'a str);

impl<'a> TryFrom<&'a str> for AtomRef<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.bytes().all(is_astring_char) {
            Ok(AtomRef(value))
        } else {
            Err(())
        }
    }
}

impl<'a> Deref for AtomRef<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> AtomRef<'a> {
    pub fn to_owned(&self) -> Atom {
        Atom(self.0.to_string())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum IStringRef<'a> {
    Literal(LiteralRef<'a>),
    Quoted(Cow<'a, str>),
}

impl<'a> IStringRef<'a> {
    pub fn to_owned(&self) -> IString {
        match self {
            IStringRef::Literal(literal_ref) => IString::Literal(Literal::from(literal_ref)),
            IStringRef::Quoted(cowstr) => IString::Quoted(Quoted(cowstr.to_string())),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct NStringRef<'a>(pub Option<IStringRef<'a>>);

impl<'a> NStringRef<'a> {
    pub fn to_owned(&self) -> NString {
        NString(self.0.as_ref().map(|inner| inner.to_owned()))
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum AStringRef<'a> {
    Atom(AtomRef<'a>),
    String(IStringRef<'a>),
}

impl<'a> AStringRef<'a> {
    pub fn to_owned(&self) -> AString {
        match self {
            AStringRef::Atom(atom) => AString::Atom(atom.to_owned()),
            AStringRef::String(istr) => AString::String(istr.to_owned()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct txt<'a>(pub(crate) &'a str);

impl<'a> txt<'a> {
    pub fn to_owned(&self) -> Text {
        Text(self.0.to_string())
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyVec<T>(pub(crate) Vec<T>);

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = ();

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(())
        } else {
            Ok(NonEmptyVec(value))
        }
    }
}

impl<T> Deref for NonEmptyVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::Encode;

    #[test]
    fn test_conversion() {
        assert_eq!(
            IString::try_from("AAA").unwrap(),
            IString::Quoted("AAA".try_into().unwrap()).into()
        );
        assert_eq!(
            IString::try_from("\"AAA").unwrap(),
            IString::Quoted("\"AAA".try_into().unwrap()).into()
        );

        assert_ne!(
            IString::try_from("\"AAA").unwrap(),
            IString::Quoted("\\\"AAA".try_into().unwrap()).into()
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
