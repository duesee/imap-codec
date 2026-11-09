//! The IMAP OBJECTID extension (RFC 8474).
//!
//! Servers that advertise `OBJECTID` give each message a stable `EMAILID`
//! (unchanged by moves and UIDVALIDITY resets) and a `THREADID`.

use alloc::{borrow::Cow, string::String};
use core::{
    fmt::{Debug, Formatter},
    str::from_utf8,
};

use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::{ValidationError, ValidationErrorKind};

/// A server-assigned object identifier.
///
/// ```abnf
/// objectid = 1*255(ALPHA / DIGIT / "_" / "-")
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, ToStatic)]
pub struct ObjectId<'a>(pub(crate) Cow<'a, str>);

impl Debug for ObjectId<'_> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "ObjectId({:?})", self.0)
    }
}

impl<'a> ObjectId<'a> {
    /// The longest identifier RFC 8474 allows.
    pub const MAX_LEN: usize = 255;

    /// Validates if value conforms to objectid's ABNF definition.
    pub fn validate(value: impl AsRef<[u8]>) -> Result<(), ValidationError> {
        let value = value.as_ref();

        if value.is_empty() {
            return Err(ValidationError::new(ValidationErrorKind::Empty));
        }

        if value.len() > Self::MAX_LEN {
            return Err(ValidationError::new(ValidationErrorKind::Invalid));
        }

        if let Some(at) = value.iter().position(|b| !is_objectid_char(*b)) {
            return Err(ValidationError::new(ValidationErrorKind::InvalidByteAt {
                byte: value[at],
                at,
            }));
        }

        Ok(())
    }

    /// Returns a reference to the inner value.
    pub fn inner(&self) -> &str {
        self.0.as_ref()
    }

    /// Consumes the identifier, returning the inner value.
    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }

    /// Constructs an identifier without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `inner` is valid according to [`Self::validate`].
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::validate(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

/// `ALPHA / DIGIT / "_" / "-"`
pub(crate) fn is_objectid_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

impl<'a> TryFrom<&'a [u8]> for ObjectId<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::validate(value)?;

        // Safety: `unwrap` can't panic due to `validate`.
        Ok(Self(Cow::Borrowed(from_utf8(value).unwrap())))
    }
}

impl<'a> TryFrom<&'a str> for ObjectId<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::validate(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl TryFrom<String> for ObjectId<'_> {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(Cow::Owned(value)))
    }
}

impl AsRef<str> for ObjectId<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_validate() {
        assert!(ObjectId::try_from("M6d99ac3275bb4e").is_ok());
        assert!(ObjectId::try_from("T-_9").is_ok());
        assert!(ObjectId::try_from("").is_err());
        assert!(ObjectId::try_from("a b").is_err());
        assert!(ObjectId::try_from("a.b").is_err());
        assert!(ObjectId::try_from("x".repeat(255).to_string()).is_ok());
        assert!(ObjectId::try_from("x".repeat(256).to_string()).is_err());
    }
}
