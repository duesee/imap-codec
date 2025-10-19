use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

/// A quoted UTF-8 string.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[derive(Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct QuotedUtf8<'a>(pub Cow<'a, str>);

impl From<String> for QuotedUtf8<'_> {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl Debug for QuotedUtf8<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "QuotedUtf8({:?})", self.0)
    }
}
