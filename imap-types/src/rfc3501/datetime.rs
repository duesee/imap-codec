use std::fmt::{Debug, Formatter};

#[cfg(feature = "bounded-static")]
use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use chrono::{DateTime, FixedOffset, NaiveDate};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MyDateTime(pub DateTime<FixedOffset>);

impl Debug for MyDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "bounded-static")]
impl IntoBoundedStatic for MyDateTime {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

#[cfg(feature = "bounded-static")]
impl ToBoundedStatic for MyDateTime {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MyNaiveDate(pub NaiveDate);

impl Debug for MyNaiveDate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "bounded-static")]
impl IntoBoundedStatic for MyNaiveDate {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

#[cfg(feature = "bounded-static")]
impl ToBoundedStatic for MyNaiveDate {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}
