use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::{ValidationError, ValidationErrorKind};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum Utf8Kind {
    Accept,
    Only,
}

impl Display for Utf8Kind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Accept => "ACCEPT",
            Self::Only => "ONLY",
        })
    }
}

/// A UTF-8 quoted string without NUL, CR, or LF.
///
/// [RFC 9755, Section 3](https://www.rfc-editor.org/rfc/rfc9755.html#section-3)
/// adds multibyte UTF-8 sequences to the `quoted` syntax from
/// [RFC 3501, Section 9](https://www.rfc-editor.org/rfc/rfc3501.html#section-9);
/// NUL, CR, and LF remain excluded.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[derive(Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct QuotedUtf8<'a>(pub(crate) Cow<'a, str>);

impl Debug for QuotedUtf8<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "QuotedUtf8({:?})", self.0)
    }
}

impl<'a> QuotedUtf8<'a> {
    pub fn validate(value: impl AsRef<str>) -> Result<(), ValidationError> {
        let value = value.as_ref().as_bytes();

        if let Some(at) = value
            .iter()
            .position(|byte| matches!(byte, 0 | b'\r' | b'\n'))
        {
            return Err(ValidationError::new(ValidationErrorKind::InvalidByteAt {
                byte: value[at],
                at,
            }));
        };

        Ok(())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }

    /// Constructs a UTF-8 quoted string without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `inner` is valid according to [`Self::validate`]. Failing to do
    /// so may create invalid/unparsable IMAP messages, or even produce unintended protocol flows.
    /// Do not call this constructor with untrusted data.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::validate(inner.as_ref()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a str> for QuotedUtf8<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::validate(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl TryFrom<String> for QuotedUtf8<'_> {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(Cow::Owned(value)))
    }
}

impl AsRef<str> for QuotedUtf8<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::QuotedUtf8;
    use crate::error::{ValidationError, ValidationErrorKind};

    #[test]
    fn test_quoted_utf8_construction() {
        for value in ["", "ASCII", "café日本語😀", "\"\\", "\t\u{7f}\u{80}"] {
            let quoted = QuotedUtf8::try_from(value).unwrap();
            assert_eq!(quoted.inner(), value);
            assert!(matches!(quoted.into_inner(), Cow::Borrowed(_)));
            let owned = QuotedUtf8::try_from(value.to_owned()).unwrap();
            assert_eq!(owned.as_ref(), value);
            assert!(matches!(owned.into_inner(), Cow::Owned(_)));
        }
        for (value, at) in [("\0", 0), ("a\rb", 1), ("café\n", 5)] {
            let expected = ValidationError::new(ValidationErrorKind::InvalidByteAt {
                byte: value.as_bytes()[at],
                at,
            });
            assert_eq!(QuotedUtf8::try_from(value), Err(expected.clone()));
            assert_eq!(QuotedUtf8::try_from(value.to_owned()), Err(expected));
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_quoted_utf8_serde() {
        let valid = QuotedUtf8::try_from("café\"\\").unwrap();
        let json = serde_json::to_string(&valid).unwrap();
        assert_eq!(serde_json::from_str::<QuotedUtf8>(&json).unwrap(), valid);
        for json in [r#""\u0000""#, r#""a\rb""#, r#""a\nb""#] {
            assert!(serde_json::from_str::<QuotedUtf8>(json).is_err());
        }
    }

    #[cfg(feature = "arbitrary")]
    #[test]
    fn test_arbitrary_quoted_utf8() {
        use arbitrary::{Arbitrary, Error, Unstructured};

        // Arbitrary consumes the final byte as the string length.
        let value = QuotedUtf8::arbitrary(&mut Unstructured::new(b"caf\xc3\xa9\x05")).unwrap();
        assert_eq!(value.inner(), "café");
        for input in [b"a\0b\x03", b"a\rb\x03", b"a\nb\x03"] {
            assert_eq!(
                QuotedUtf8::arbitrary(&mut Unstructured::new(input)),
                Err(Error::IncorrectFormat),
            );
        }
    }
}
