use std::fmt::{Debug, Formatter};

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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MyNaiveDate(pub NaiveDate);

impl Debug for MyNaiveDate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
