use std::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{core::Atom, error::ValidationError};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum AttributeFlag<'a> {
    Answered,
    Deleted,
    Draft,
    Flagged,
    Seen,
    Extension(AttributeFlagExtension<'a>),
    Keyword(Atom<'a>),
}

impl<'a> AttributeFlag<'a> {
    pub fn system(atom: Atom<'a>) -> Self {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "answered" => Self::Answered,
            "flagged" => Self::Flagged,
            "deleted" => Self::Deleted,
            "seen" => Self::Seen,
            "draft" => Self::Draft,
            _ => Self::Extension(AttributeFlagExtension(atom)),
        }
    }

    pub fn keyword(atom: Atom<'a>) -> Self {
        Self::Keyword(atom)
    }
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct AttributeFlagExtension<'a>(Atom<'a>);

impl<'a> TryFrom<&'a str> for AttributeFlag<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(if let Some(value) = value.strip_prefix("\\\\") {
            Self::system(Atom::try_from(value)?)
        } else {
            Self::keyword(Atom::try_from(value)?)
        })
    }
}

impl Display for AttributeFlag<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AttributeFlag::Answered => f.write_str("\\\\Answered"),
            AttributeFlag::Flagged => f.write_str("\\\\Flagged"),
            AttributeFlag::Deleted => f.write_str("\\\\Deleted"),
            AttributeFlag::Seen => f.write_str("\\\\Seen"),
            AttributeFlag::Draft => f.write_str("\\\\Draft"),
            AttributeFlag::Keyword(atom) => write!(f, "{atom}"),
            AttributeFlag::Extension(other) => write!(f, "\\\\{}", other.0),
        }
    }
}

#[cfg(feature = "ext_condstore_qresync")]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum EntryTypeReq {
    Private,
    Shared,
    All,
}

#[cfg(feature = "ext_condstore_qresync")]
impl Display for EntryTypeReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryTypeReq::Private => write!(f, "priv"),
            EntryTypeReq::Shared => write!(f, "shared"),
            EntryTypeReq::All => write!(f, "all"),
        }
    }
}
