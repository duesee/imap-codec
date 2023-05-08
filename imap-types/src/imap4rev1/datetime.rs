use std::fmt::{Debug, Formatter};

#[cfg(feature = "bounded-static")]
use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use chrono::{DateTime as ChronoDateTime, FixedOffset, NaiveDate as ChronoNaiveDate};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DateTime(pub ChronoDateTime<FixedOffset>);

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "bounded-static")]
impl IntoBoundedStatic for DateTime {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

#[cfg(feature = "bounded-static")]
impl ToBoundedStatic for DateTime {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct NaiveDate(pub ChronoNaiveDate);

impl Debug for NaiveDate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "bounded-static")]
impl IntoBoundedStatic for NaiveDate {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

#[cfg(feature = "bounded-static")]
impl ToBoundedStatic for NaiveDate {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}
