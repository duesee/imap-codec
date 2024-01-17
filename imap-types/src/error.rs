//! Error-related types.

use std::fmt::{Display, Formatter};

use thiserror::Error;

/// A validation error.
///
/// This error can be returned during validation of a value, e.g., a tag, atom, etc.
#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidationError {
    kind: ValidationErrorKind,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Validation failed: {}", self.kind)
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ValidationErrorKind {
    #[error("Must not be empty")]
    Empty,
    #[error("Must have at least {min} elements")]
    NotEnough { min: usize },
    #[error("Invalid value")]
    Invalid,
    #[error("Invalid byte b'\\x{byte:02x}' at index {at}")]
    InvalidByteAt { byte: u8, at: usize },
}

impl ValidationError {
    pub(crate) fn new(kind: ValidationErrorKind) -> Self {
        Self { kind }
    }
}
