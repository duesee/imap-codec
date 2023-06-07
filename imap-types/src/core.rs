//! Core data types
//!
//! This module exposes imap-types' "core types" (or "string types").
//!
//! # Overview
//!
//! ```text
//!        ┌───────┐ ┌─────────────────┐
//!        │AString│ │     NString     │
//!        └──┬─┬──┘ │(Option<IString>)│
//!           │ │    └─────┬───────────┘
//!           │ └──────┐   │
//!           │        │   │
//! ┌────┐ ┌──▼────┐ ┌─▼───▼─┐
//! │Atom│ │AtomExt│ │IString│
//! └────┘ └───────┘ └┬─────┬┘
//!                   │     │
//!             ┌─────▼─┐ ┌─▼────┐
//!             │Literal│ │Quoted│
//!             └───────┘ └──────┘
//! ```

use std::{borrow::Cow, str::from_utf8, vec::IntoIter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::utils::indicators::{
    is_any_text_char_except_quoted_specials, is_astring_char, is_atom_char, is_char8, is_text_char,
};

macro_rules! impl_try_from {
    ($via:ty, $lifetime:lifetime, $from:ty, $target:ty) => {
        impl<$lifetime> TryFrom<$from> for $target {
            type Error = <$via as TryFrom<$from>>::Error;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let value = <$via>::try_from(value)?;

                Ok(Self::from(value))
            }
        }
    };
}

pub(crate) use impl_try_from;

/// An atom.
///
/// "An atom consists of one or more non-special characters." ([RFC 3501](https://www.rfc-editor.org/rfc/rfc3501.html))
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Atom<'a>(pub(crate) Cow<'a, str>);

impl<'a> Atom<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), AtomError> {
        let value = value.as_ref();

        if value.is_empty() {
            return Err(AtomError::Empty);
        }

        if let Some(position) = value.iter().position(|b| !is_atom_char(*b)) {
            return Err(AtomError::ByteNotAllowed {
                found: value[position],
                position,
            });
        };

        Ok(())
    }

    pub fn inner(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }

    #[cfg(feature = "unchecked")]
    pub fn unchecked<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::verify(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a [u8]> for Atom<'a> {
    type Error = AtomError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Self(Cow::Borrowed(from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<Vec<u8>> for Atom<'a> {
    type Error = AtomError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Self(Cow::Owned(String::from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<&'a str> for Atom<'a> {
    type Error = AtomError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl<'a> TryFrom<String> for Atom<'a> {
    type Error = AtomError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        Ok(Atom(Cow::Owned(value)))
    }
}

impl<'a> TryFrom<Cow<'a, str>> for Atom<'a> {
    type Error = AtomError;

    fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
        Self::verify(value.as_bytes())?;

        Ok(Atom(value))
    }
}

impl<'a> AsRef<str> for Atom<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum AtomError {
    #[error("Must not be empty")]
    Empty,
    #[error("Invalid byte b'\\x{found:02x}' at index {position}")]
    ByteNotAllowed { found: u8, position: usize },
}

/// An (extended) atom.
///
/// According to IMAP's formal syntax, an atom with additional allowed chars.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomExt<'a>(pub(crate) Cow<'a, str>);

impl<'a> AtomExt<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), AtomExtError> {
        let value = value.as_ref();

        if value.is_empty() {
            return Err(AtomExtError::Empty);
        }

        if let Some(position) = value.iter().position(|b| !is_astring_char(*b)) {
            return Err(AtomExtError::ByteNotAllowed {
                found: value[position],
                position,
            });
        };

        Ok(())
    }

    pub fn inner(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }

    #[cfg(feature = "unchecked")]
    pub fn unchecked<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::verify(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a [u8]> for AtomExt<'a> {
    type Error = AtomExtError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Self(Cow::Borrowed(from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<Vec<u8>> for AtomExt<'a> {
    type Error = AtomExtError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Self(Cow::Owned(String::from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<&'a str> for AtomExt<'a> {
    type Error = AtomExtError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl<'a> TryFrom<String> for AtomExt<'a> {
    type Error = AtomExtError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        Ok(Self(Cow::Owned(value)))
    }
}

impl<'a> From<Atom<'a>> for AtomExt<'a> {
    fn from(value: Atom<'a>) -> Self {
        Self(value.0)
    }
}

impl<'a> AsRef<str> for AtomExt<'a> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum AtomExtError {
    #[error("Must not be empty.")]
    Empty,
    #[error("Invalid byte b'\\x{found:02x}' at index {position}")]
    ByteNotAllowed { found: u8, position: usize },
}

// ## 4.2. Number
//
// A number consists of one or more digit characters, and
// represents a numeric value.

// ## 4.3. String

/// Either a literal or a quoted string.
///
/// "The empty string is represented as either "" (a quoted string with zero characters between double quotes) or as {0} followed by CRLF (a literal with an octet count of 0)." ([RFC 3501](https://www.rfc-editor.org/rfc/rfc3501.html))
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IString<'a> {
    Literal(Literal<'a>),
    Quoted(Quoted<'a>),
}

impl<'a> IString<'a> {
    pub fn into_inner(self) -> Cow<'a, [u8]> {
        match self {
            Self::Literal(literal) => literal.into_inner(),
            Self::Quoted(quoted) => match quoted.into_inner() {
                Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
                Cow::Owned(s) => Cow::Owned(s.into_bytes()),
            },
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for IString<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if let Ok(quoted) = Quoted::try_from(value) {
            return Ok(IString::Quoted(quoted));
        }

        Ok(IString::Literal(Literal::try_from(value)?))
    }
}

impl TryFrom<Vec<u8>> for IString<'_> {
    type Error = LiteralError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO(efficiency)
        if let Ok(quoted) = Quoted::try_from(value.clone()) {
            return Ok(IString::Quoted(quoted));
        }

        Ok(IString::Literal(Literal::try_from(value)?))
    }
}

impl<'a> TryFrom<&'a str> for IString<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Ok(quoted) = Quoted::try_from(value) {
            return Ok(IString::Quoted(quoted));
        }

        Ok(IString::Literal(Literal::try_from(value)?))
    }
}

impl<'a> TryFrom<String> for IString<'a> {
    type Error = LiteralError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO(efficiency)
        if let Ok(quoted) = Quoted::try_from(value.clone()) {
            return Ok(IString::Quoted(quoted));
        }

        Ok(IString::Literal(Literal::try_from(value)?))
    }
}

impl<'a> From<Literal<'a>> for IString<'a> {
    fn from(value: Literal<'a>) -> Self {
        Self::Literal(value)
    }
}

impl<'a> From<Quoted<'a>> for IString<'a> {
    fn from(value: Quoted<'a>) -> Self {
        Self::Quoted(value)
    }
}

impl<'a> AsRef<[u8]> for IString<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Literal(literal) => literal.as_ref(),
            Self::Quoted(quoted) => quoted.as_ref().as_bytes(),
        }
    }
}

/// A literal.
///
/// "A literal is a sequence of zero or more octets (including CR and LF), prefix-quoted with an octet count in the form of an open brace ("{"), the number of octets, close brace ("}"), and CRLF.
/// In the case of literals transmitted from server to client, the CRLF is immediately followed by the octet data.
/// In the case of literals transmitted from client to server, the client MUST wait to receive a command continuation request (...) before sending the octet data (and the remainder of the command).
///
/// Note: Even if the octet count is 0, a client transmitting a literal MUST wait to receive a command continuation request." ([RFC 3501](https://www.rfc-editor.org/rfc/rfc3501.html))
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal<'a> {
    pub(crate) data: Cow<'a, [u8]>,
    #[cfg(feature = "ext_literal")]
    /// Specifies whether this is a synchronizing or non-synchronizing literal.
    ///
    /// `true` (default) denotes a synchronizing literal, e.g., `{3}\r\nfoo`.
    /// `false` denotes a non-synchronizing literal, e.g., `{3+}\r\nfoo`.
    ///
    /// Note: In the special case that a server advertised a `LITERAL-` capability, AND the literal
    /// has more than 4096 bytes a non-synchronizing literal must still be treated as synchronizing.
    pub sync: bool,
}

impl<'a> Literal<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), LiteralError> {
        let value = value.as_ref();

        if let Some(position) = value.iter().position(|b| !is_char8(*b)) {
            return Err(LiteralError::ByteNotAllowed {
                found: value[position],
                position,
            });
        };

        Ok(())
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    #[cfg(feature = "ext_literal")]
    pub fn into_sync(mut self) -> Self {
        self.sync = true;
        self
    }

    #[cfg(feature = "ext_literal")]
    pub fn into_non_sync(mut self) -> Self {
        self.sync = false;
        self
    }

    pub fn into_inner(self) -> Cow<'a, [u8]> {
        self.data
    }

    /// Create a literal from a byte sequence without checking
    /// that it conforms to IMAP's literal specification.
    ///
    /// # Safety
    ///
    /// Call this function only when you are sure that the byte sequence
    /// is a valid literal, i.e., that it does not contain 0x00.
    #[cfg(feature = "unchecked")]
    pub fn unchecked<D>(data: D, #[cfg(feature = "ext_literal")] sync: bool) -> Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        let data = data.into();

        #[cfg(debug_assertions)]
        Self::verify(&data).unwrap();

        Self {
            data,
            #[cfg(feature = "ext_literal")]
            sync,
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Literal<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(Literal {
            data: Cow::Borrowed(value),
            #[cfg(feature = "ext_literal")]
            sync: true,
        })
    }
}

impl<'a> TryFrom<Vec<u8>> for Literal<'a> {
    type Error = LiteralError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        Ok(Literal {
            data: Cow::Owned(value),
            #[cfg(feature = "ext_literal")]
            sync: true,
        })
    }
}

impl<'a> TryFrom<&'a str> for Literal<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(Literal {
            data: Cow::Borrowed(value.as_bytes()),
            #[cfg(feature = "ext_literal")]
            sync: true,
        })
    }
}

impl<'a> TryFrom<String> for Literal<'a> {
    type Error = LiteralError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        Ok(Literal {
            data: Cow::Owned(value.into_bytes()),
            #[cfg(feature = "ext_literal")]
            sync: true,
        })
    }
}

impl<'a> AsRef<[u8]> for Literal<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiteralError {
    #[error("Invalid byte b'\\x{found:02x}' at index {position}")]
    ByteNotAllowed { found: u8, position: usize },
}

/// A quoted string.
///
/// "The quoted string form is an alternative that avoids the overhead of processing a literal at the cost of limitations of characters which may be used.
///
/// A quoted string is a sequence of zero or more 7-bit characters, excluding CR and LF, with double quote (<">) characters at each end." ([RFC 3501](https://www.rfc-editor.org/rfc/rfc3501.html))
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quoted<'a>(pub(crate) Cow<'a, str>);

impl<'a> Quoted<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), QuotedError> {
        let value = value.as_ref();

        if let Some(position) = value.iter().position(|b| !is_text_char(*b)) {
            return Err(QuotedError::ByteNotAllowed {
                found: value[position],
                position,
            });
        };

        Ok(())
    }

    pub fn inner(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }

    /// Create a quoted from a string without checking
    /// that it to conforms to IMAP's quoted specification.
    ///
    /// # Safety
    ///
    /// Call this function only when you are sure that the str
    /// is a valid quoted.
    #[cfg(feature = "unchecked")]
    pub fn unchecked<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::verify(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a [u8]> for Quoted<'a> {
    type Error = QuotedError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Quoted::verify(value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Quoted(Cow::Borrowed(from_utf8(value).unwrap())))
    }
}

impl TryFrom<Vec<u8>> for Quoted<'_> {
    type Error = QuotedError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Quoted::verify(&value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Quoted(Cow::Owned(String::from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<&'a str> for Quoted<'a> {
    type Error = QuotedError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Quoted::verify(value)?;

        Ok(Quoted(Cow::Borrowed(value)))
    }
}

impl<'a> TryFrom<String> for Quoted<'a> {
    type Error = QuotedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Quoted::verify(&value)?;

        Ok(Quoted(Cow::Owned(value)))
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum QuotedError {
    #[error("Invalid byte b'\\x{found:02x}' at index {position}")]
    ByteNotAllowed { found: u8, position: usize },
}

impl<'a> AsRef<str> for Quoted<'a> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Either NIL or a string.
///
/// This is modeled using Rust's `Option` type.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NString<'a>(
    // This wrapper is merely used for formatting.
    // The inner value can be public.
    pub Option<IString<'a>>,
);

impl<'a> NString<'a> {
    pub fn into_option(self) -> Option<Cow<'a, [u8]>> {
        self.0.map(|inner| inner.into_inner())
    }
}

impl<'a> TryFrom<&'a str> for NString<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(Some(IString::try_from(value)?)))
    }
}

impl<'a> From<Literal<'a>> for NString<'a> {
    fn from(value: Literal<'a>) -> Self {
        Self(Some(IString::from(value)))
    }
}

impl<'a> From<Quoted<'a>> for NString<'a> {
    fn from(value: Quoted<'a>) -> Self {
        Self(Some(IString::from(value)))
    }
}

/// Either an (extended) atom or a string.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AString<'a> {
    // `1*ATOM-CHAR` does not allow resp-specials, but `1*ASTRING-CHAR` does ... :-/
    Atom(AtomExt<'a>),   // 1*ASTRING-CHAR /
    String(IString<'a>), // string
}

impl<'a> TryFrom<&'a [u8]> for AString<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if let Ok(atom) = AtomExt::try_from(value) {
            return Ok(AString::Atom(atom));
        }

        Ok(AString::String(IString::try_from(value)?))
    }
}

impl TryFrom<Vec<u8>> for AString<'_> {
    type Error = LiteralError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO(efficiency)
        if let Ok(atom) = AtomExt::try_from(value.clone()) {
            return Ok(AString::Atom(atom));
        }

        Ok(AString::String(IString::try_from(value)?))
    }
}

impl<'a> TryFrom<&'a str> for AString<'a> {
    type Error = LiteralError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Ok(atom) = AtomExt::try_from(value) {
            return Ok(AString::Atom(atom));
        }

        Ok(AString::String(IString::try_from(value)?))
    }
}

impl<'a> TryFrom<String> for AString<'a> {
    type Error = LiteralError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO(efficiency)
        if let Ok(atom) = AtomExt::try_from(value.clone()) {
            return Ok(AString::Atom(atom));
        }

        Ok(AString::String(IString::try_from(value)?))
    }
}

impl<'a> From<Atom<'a>> for AString<'a> {
    fn from(atom: Atom<'a>) -> Self {
        AString::Atom(AtomExt::from(atom))
    }
}

impl<'a> From<AtomExt<'a>> for AString<'a> {
    fn from(atom: AtomExt<'a>) -> Self {
        AString::Atom(atom)
    }
}

impl<'a> From<Quoted<'a>> for AString<'a> {
    fn from(value: Quoted<'a>) -> Self {
        AString::String(IString::Quoted(value))
    }
}

impl<'a> From<Literal<'a>> for AString<'a> {
    fn from(value: Literal<'a>) -> Self {
        AString::String(IString::Literal(value))
    }
}

impl<'a> AsRef<[u8]> for AString<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Atom(atom_ext) => atom_ext.as_ref().as_bytes(),
            Self::String(istr) => istr.as_ref(),
        }
    }
}

// 4.3.1.  8-bit and Binary Strings
//
//    8-bit textual and binary mail is supported through the use of a
//    [MIME-IMB] content transfer encoding.  IMAP4rev1 implementations MAY
//    transmit 8-bit or multi-octet characters in literals, but SHOULD do
//    so only when the \[CHARSET\] is identified.
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

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Tag<'a>(pub(crate) Cow<'a, str>);

impl<'a> Tag<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), TagError> {
        let value = value.as_ref();

        if value.is_empty() {
            return Err(TagError::Empty);
        }

        if let Some(position) = value
            .iter()
            .position(|b| !is_astring_char(*b) || *b == b'+')
        {
            return Err(TagError::ByteNotAllowed {
                found: value[position],
                position,
            });
        };

        Ok(())
    }

    pub fn inner(&self) -> &str {
        self.0.as_ref()
    }

    #[cfg(feature = "unchecked")]
    pub fn unchecked<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::verify(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a [u8]> for Tag<'a> {
    type Error = TagError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        // Safety: `unwrap` can't fail due to `verify`.
        Ok(Self(Cow::Borrowed(from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<Vec<u8>> for Tag<'a> {
    type Error = TagError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        // Safety: `unwrap` can't fail due to `verify`.
        Ok(Self(Cow::Owned(String::from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<&'a str> for Tag<'a> {
    type Error = TagError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl<'a> TryFrom<String> for Tag<'a> {
    type Error = TagError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        Ok(Self(Cow::Owned(value)))
    }
}

impl<'a> AsRef<str> for Tag<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum TagError {
    #[error("Must not be empty.")]
    Empty,
    #[error("Invalid byte b'\\x{found:02x}' at index {position}")]
    ByteNotAllowed { found: u8, position: usize },
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Text<'a>(pub(crate) Cow<'a, str>);

impl<'a> Text<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), TextError> {
        let value = value.as_ref();

        if value.is_empty() {
            return Err(TextError::Empty);
        }

        if let Some(position) = value.iter().position(|b| !is_text_char(*b)) {
            return Err(TextError::ByteNotAllowed {
                found: value[position],
                position,
            });
        };

        Ok(())
    }

    pub fn inner(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }

    #[cfg(feature = "unchecked")]
    pub fn unchecked<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::verify(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a [u8]> for Text<'a> {
    type Error = TextError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Self(Cow::Borrowed(from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<Vec<u8>> for Text<'a> {
    type Error = TextError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        // Safety: `unwrap` can't panic due to `verify`.
        Ok(Self(Cow::Owned(String::from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<&'a str> for Text<'a> {
    type Error = TextError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl<'a> TryFrom<String> for Text<'a> {
    type Error = TextError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::verify(&value)?;

        Ok(Self(Cow::Owned(value)))
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextError {
    #[error("Must not be empty.")]
    Empty,
    #[error("Invalid byte b'\\x{found:02x}' at index {position}")]
    ByteNotAllowed { found: u8, position: usize },
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Debug, PartialEq, Eq, Hash, Clone)]
pub struct QuotedChar(char);

impl QuotedChar {
    pub fn verify(input: char) -> Result<(), QuotedCharError> {
        if input.is_ascii()
            && (is_any_text_char_except_quoted_specials(input as u8)
                || input == '\\'
                || input == '"')
        {
            Ok(())
        } else {
            Err(QuotedCharError::Invalid(input))
        }
    }

    pub fn inner(&self) -> char {
        self.0
    }

    #[cfg(feature = "unchecked")]
    pub fn unchecked(inner: char) -> Self {
        #[cfg(debug_assertions)]
        Self::verify(inner).unwrap();

        Self(inner)
    }
}

impl TryFrom<char> for QuotedChar {
    type Error = QuotedCharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::verify(value)?;

        Ok(QuotedChar(value))
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum QuotedCharError {
    #[error("Invalid character `{0}`.")]
    Invalid(char),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Charset<'a> {
    Atom(Atom<'a>),
    Quoted(Quoted<'a>),
}

impl<'a> From<Atom<'a>> for Charset<'a> {
    fn from(value: Atom<'a>) -> Self {
        Self::Atom(value)
    }
}

impl<'a> From<Quoted<'a>> for Charset<'a> {
    fn from(value: Quoted<'a>) -> Self {
        Self::Quoted(value)
    }
}

impl<'a> TryFrom<&'a [u8]> for Charset<'a> {
    type Error = QuotedError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if let Ok(atom) = Atom::try_from(value) {
            return Ok(Self::Atom(atom));
        }

        Ok(Self::Quoted(Quoted::try_from(value)?))
    }
}

impl<'a> TryFrom<Vec<u8>> for Charset<'a> {
    type Error = QuotedError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO(efficiency)
        if let Ok(atom) = Atom::try_from(value.clone()) {
            return Ok(Self::Atom(atom));
        }

        Ok(Self::Quoted(Quoted::try_from(value)?))
    }
}

impl<'a> TryFrom<&'a str> for Charset<'a> {
    type Error = QuotedError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Ok(atom) = Atom::try_from(value) {
            return Ok(Self::Atom(atom));
        }

        Ok(Self::Quoted(Quoted::try_from(value)?))
    }
}

impl<'a> TryFrom<String> for Charset<'a> {
    type Error = QuotedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO(efficiency)
        if let Ok(atom) = Atom::try_from(value.clone()) {
            return Ok(Self::Atom(atom));
        }

        Ok(Self::Quoted(Quoted::try_from(value)?))
    }
}

impl<'a> AsRef<str> for Charset<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Atom(atom) => atom.as_ref(),
            Self::Quoted(quoted) => quoted.as_ref(),
        }
    }
}

/// A `Vec` that always contains >= 1 elements.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyVec<T>(pub(crate) Vec<T>);

impl<T> NonEmptyVec<T> {
    pub fn verify(value: &[T]) -> Result<(), NonEmptyVecError> {
        if value.is_empty() {
            return Err(NonEmptyVecError::Empty);
        }

        Ok(())
    }

    #[cfg(feature = "unchecked")]
    pub fn unchecked(inner: Vec<T>) -> Self {
        #[cfg(debug_assertions)]
        Self::verify(&inner).unwrap();

        Self(inner)
    }

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<T> for NonEmptyVec<T> {
    fn from(value: T) -> Self {
        NonEmptyVec(vec![value])
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = NonEmptyVecError;

    fn try_from(inner: Vec<T>) -> Result<Self, Self::Error> {
        Self::verify(&inner)?;

        Ok(Self(inner))
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum NonEmptyVecError {
    #[error("Must not be empty.")]
    Empty,
}

impl<T> AsRef<[T]> for NonEmptyVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;

    #[test]
    fn test_conversion_atom() {
        #[allow(clippy::type_complexity)]
        let tests: Vec<(&[u8], (Result<Atom, AtomError>, Result<Atom, AtomError>))> = vec![
            (
                b"A",
                (
                    Ok(Atom(Cow::Borrowed("A"))),
                    Ok(Atom(Cow::Owned("A".into()))),
                ),
            ),
            (
                b"ABC",
                (
                    Ok(Atom(Cow::Borrowed("ABC"))),
                    Ok(Atom(Cow::Owned("ABC".into()))),
                ),
            ),
            (
                b" A",
                (
                    Err(AtomError::ByteNotAllowed {
                        found: b' ',
                        position: 0,
                    }),
                    Err(AtomError::ByteNotAllowed {
                        found: b' ',
                        position: 0,
                    }),
                ),
            ),
            (
                b"A ",
                (
                    Err(AtomError::ByteNotAllowed {
                        found: b' ',
                        position: 1,
                    }),
                    Err(AtomError::ByteNotAllowed {
                        found: b' ',
                        position: 1,
                    }),
                ),
            ),
            (b"", (Err(AtomError::Empty), Err(AtomError::Empty))),
            (
                b"A\x00",
                (
                    Err(AtomError::ByteNotAllowed {
                        found: 0x00,
                        position: 1,
                    }),
                    Err(AtomError::ByteNotAllowed {
                        found: 0x00,
                        position: 1,
                    }),
                ),
            ),
            (
                b"A\x00",
                (
                    Err(AtomError::ByteNotAllowed {
                        found: 0x00,
                        position: 1,
                    }),
                    Err(AtomError::ByteNotAllowed {
                        found: 0x00,
                        position: 1,
                    }),
                ),
            ),
        ];

        for (test, (expected, expected_owned)) in tests.into_iter() {
            let got = Atom::try_from(test);
            assert_eq!(expected, got);
            if let Ok(got) = got {
                assert_eq!(got.as_ref().as_bytes(), test);
            }

            let got = Atom::try_from(test.to_owned());
            assert_eq!(expected_owned, got);
            if let Ok(got) = got {
                assert_eq!(got.as_ref().as_bytes(), test);
            }

            if let Ok(test_str) = from_utf8(test) {
                let got = Atom::try_from(test_str);
                assert_eq!(expected, got);
                if let Ok(got) = got {
                    assert_eq!(got.as_ref().as_bytes(), test);
                }

                let got = Atom::try_from(test_str.to_owned());
                assert_eq!(expected_owned, got);
                if let Ok(got) = got {
                    assert_eq!(got.as_ref().as_bytes(), test);
                }
            }
        }
    }

    #[test]
    fn test_conversion_atom_ext() {
        #[allow(clippy::type_complexity)]
        let tests: Vec<(
            &[u8],
            (Result<AtomExt, AtomExtError>, Result<AtomExt, AtomExtError>),
        )> = vec![
            (
                b"A",
                (
                    Ok(AtomExt(Cow::Borrowed("A"))),
                    Ok(AtomExt(Cow::Owned("A".into()))),
                ),
            ),
            (
                b"ABC",
                (
                    Ok(AtomExt(Cow::Borrowed("ABC"))),
                    Ok(AtomExt(Cow::Owned("ABC".into()))),
                ),
            ),
            (
                b"!partition/sda4",
                (
                    Ok(AtomExt(Cow::Borrowed("!partition/sda4"))),
                    Ok(AtomExt(Cow::Owned("!partition/sda4".into()))),
                ),
            ),
            (
                b" A",
                (
                    Err(AtomExtError::ByteNotAllowed {
                        found: b' ',
                        position: 0,
                    }),
                    Err(AtomExtError::ByteNotAllowed {
                        found: b' ',
                        position: 0,
                    }),
                ),
            ),
            (
                b"A ",
                (
                    Err(AtomExtError::ByteNotAllowed {
                        found: b' ',
                        position: 1,
                    }),
                    Err(AtomExtError::ByteNotAllowed {
                        found: b' ',
                        position: 1,
                    }),
                ),
            ),
            (b"", (Err(AtomExtError::Empty), Err(AtomExtError::Empty))),
            (
                b"A\x00",
                (
                    Err(AtomExtError::ByteNotAllowed {
                        found: 0x00,
                        position: 1,
                    }),
                    Err(AtomExtError::ByteNotAllowed {
                        found: 0x00,
                        position: 1,
                    }),
                ),
            ),
            (
                b"\x00",
                (
                    Err(AtomExtError::ByteNotAllowed {
                        found: 0x00,
                        position: 0,
                    }),
                    Err(AtomExtError::ByteNotAllowed {
                        found: 0x00,
                        position: 0,
                    }),
                ),
            ),
        ];

        for (test, (expected, expected_owned)) in tests.into_iter() {
            let got = AtomExt::try_from(test);
            assert_eq!(expected, got);
            if let Ok(got) = got {
                assert_eq!(got.as_ref().as_bytes(), test);
            }

            let got = AtomExt::try_from(test.to_owned());
            assert_eq!(expected_owned, got);
            if let Ok(got) = got {
                assert_eq!(got.as_ref().as_bytes(), test);
            }

            if let Ok(test_str) = from_utf8(test) {
                let got = AtomExt::try_from(test_str);
                assert_eq!(expected, got);
                if let Ok(got) = got {
                    assert_eq!(got.as_ref().as_bytes(), test);
                }

                let got = AtomExt::try_from(test_str.to_owned());
                assert_eq!(expected_owned, got);
                if let Ok(got) = got {
                    assert_eq!(got.as_ref().as_bytes(), test);
                }
            }
        }
    }

    #[test]
    fn test_conversion_astring() {
        #[allow(clippy::type_complexity)]
        let tests: Vec<(
            &[u8],
            (Result<AString, LiteralError>, Result<AString, LiteralError>),
        )> = vec![
            (
                b"A",
                (
                    Ok(AString::Atom(AtomExt(Cow::Borrowed("A")))),
                    Ok(AString::Atom(AtomExt(Cow::Owned("A".into())))),
                ),
            ),
            (
                b"ABC",
                (
                    Ok(AString::Atom(AtomExt(Cow::Borrowed("ABC")))),
                    Ok(AString::Atom(AtomExt(Cow::Owned("ABC".into())))),
                ),
            ),
            (
                b"",
                (
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Borrowed(""))))),
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Owned(
                        "".to_owned(),
                    ))))),
                ),
            ),
            (
                b" A",
                (
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Borrowed(
                        " A",
                    ))))),
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Owned(
                        " A".to_owned(),
                    ))))),
                ),
            ),
            (
                b"A ",
                (
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Borrowed(
                        "A ",
                    ))))),
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Owned(
                        "A ".to_owned(),
                    ))))),
                ),
            ),
            (
                b"\"",
                (
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Borrowed(
                        "\"",
                    ))))),
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Owned(
                        "\"".to_owned(),
                    ))))),
                ),
            ),
            (
                b"\\\"",
                (
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Borrowed(
                        "\\\"",
                    ))))),
                    Ok(AString::String(IString::Quoted(Quoted(Cow::Owned(
                        "\\\"".to_owned(),
                    ))))),
                ),
            ),
            (
                b"A\x00",
                (
                    Err(LiteralError::ByteNotAllowed {
                        found: 0,
                        position: 1,
                    }),
                    Err(LiteralError::ByteNotAllowed {
                        found: 0,
                        position: 1,
                    }),
                ),
            ),
            (
                b"\x00",
                (
                    Err(LiteralError::ByteNotAllowed {
                        found: 0,
                        position: 0,
                    }),
                    Err(LiteralError::ByteNotAllowed {
                        found: 0,
                        position: 0,
                    }),
                ),
            ),
        ];

        for (test, (expected, expected_owned)) in tests.into_iter() {
            let got = AString::try_from(test);
            assert_eq!(expected, got);
            if let Ok(got) = got {
                assert_eq!(got.as_ref(), test);
            }

            let got = AString::try_from(test.to_owned());
            assert_eq!(expected_owned, got);
            if let Ok(got) = got {
                assert_eq!(got.as_ref(), test);
            }

            if let Ok(test_str) = from_utf8(test) {
                let got = AString::try_from(test_str);
                assert_eq!(expected, got);
                if let Ok(got) = got {
                    assert_eq!(got.as_ref(), test);
                }

                let got = AString::try_from(test_str.to_owned());
                assert_eq!(expected_owned, got);
                if let Ok(got) = got {
                    assert_eq!(got.as_ref(), test);
                }
            }
        }
    }

    #[test]
    fn test_conversion_istring() {
        assert_eq!(
            IString::try_from("AAA").unwrap(),
            IString::Quoted("AAA".try_into().unwrap())
        );
        assert_eq!(
            IString::try_from("\"AAA").unwrap(),
            IString::Quoted("\"AAA".try_into().unwrap())
        );

        assert_ne!(
            IString::try_from("\"AAA").unwrap(),
            IString::Quoted("\\\"AAA".try_into().unwrap())
        );
    }
}
