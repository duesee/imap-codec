//! The IMAP COMPRESS Extension
//!
//! This extension defines a new type ...
//!
//! * [`CompressionAlgorithm`]
//!
//! ... and extends ...
//!
//! * the [`Capability`](crate::response::Capability) enum with a new variant [`Capability::Compress`](crate::response::Capability#variant.Compress),
//! * the [`Command`](crate::command::Command) enum with a new variant [`Command::Compress`](crate::command::Command#variant.Compress), and
//! * the [`Code`](crate::response::Code) enum with a new variant [`Code::CompressionActive`](crate::response::Code#variant.CompressionActive).

use std::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    command::CommandBody,
    core::Atom,
    error::{ValidationError, ValidationErrorKind},
};

impl<'a> CommandBody<'a> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the COMPRESS* capability.
    /// </div>
    pub fn compress(algorithm: CompressionAlgorithm) -> Self {
        CommandBody::Compress { algorithm }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum CompressionAlgorithm {
    Deflate,
}

impl Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Deflate => "DEFLATE",
        })
    }
}

impl<'a> TryFrom<&'a str> for CompressionAlgorithm {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_ref() {
            "deflate" => Ok(Self::Deflate),
            _ => Err(ValidationError::new(ValidationErrorKind::Invalid)),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for CompressionAlgorithm {
    type Error = ValidationError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_slice() {
            b"deflate" => Ok(Self::Deflate),
            _ => Err(ValidationError::new(ValidationErrorKind::Invalid)),
        }
    }
}

impl<'a> TryFrom<Atom<'a>> for CompressionAlgorithm {
    type Error = ValidationError;

    fn try_from(atom: Atom<'a>) -> Result<Self, Self::Error> {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "deflate" => Ok(Self::Deflate),
            _ => Err(ValidationError::new(ValidationErrorKind::Invalid)),
        }
    }
}

impl AsRef<str> for CompressionAlgorithm {
    fn as_ref(&self) -> &str {
        match self {
            CompressionAlgorithm::Deflate => "DEFLATE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let tests = [(CompressionAlgorithm::Deflate, "DEFLATE")];

        for (object, string) in tests {
            // Create from `&[u8]`.
            let got = CompressionAlgorithm::try_from(string.as_bytes()).unwrap();
            assert_eq!(object, got);

            // Create from `&str`.
            let got = CompressionAlgorithm::try_from(string).unwrap();
            assert_eq!(object, got);

            // Create from `Atom`.
            let got = CompressionAlgorithm::try_from(Atom::try_from(string).unwrap()).unwrap();
            assert_eq!(object, got);

            // AsRef
            let encoded = object.as_ref();
            assert_eq!(encoded, string);
        }
    }

    #[test]
    fn test_conversion_failing() {
        let tests = [
            "", "D", "DE", "DEF", "DEFL", "DEFLA", "DEFLAT", "DEFLATX", "DEFLATEX", "XDEFLATE",
        ];

        for string in tests {
            // Create from `&[u8]`.
            assert!(CompressionAlgorithm::try_from(string.as_bytes()).is_err());

            // Create from `&str`.
            assert!(CompressionAlgorithm::try_from(string).is_err());

            if !string.is_empty() {
                // Create from `Atom`.
                assert!(CompressionAlgorithm::try_from(Atom::try_from(string).unwrap()).is_err());
            }
        }
    }
}
