#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SortCriterion {
    pub reverse: bool,
    pub key: SortKey,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SortKey {
    Arrival,
    Cc,
    Date,
    From,
    Size,
    Subject,
    To,
}

impl AsRef<str> for SortKey {
    fn as_ref(&self) -> &str {
        match self {
            SortKey::Arrival => "ARRIVAL",
            SortKey::Cc => "CC",
            SortKey::Date => "DATE",
            SortKey::From => "FROM",
            SortKey::Size => "SIZE",
            SortKey::Subject => "SUBJECT",
            SortKey::To => "TO",
        }
    }
}
