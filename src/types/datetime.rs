use std::fmt::{Debug, Display, Formatter};

use chrono::{DateTime, FixedOffset, NaiveDate};
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MyDateTime(pub(crate) DateTime<FixedOffset>);

impl Debug for MyDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for MyDateTime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MyNaiveDate(pub(crate) NaiveDate);

impl Debug for MyNaiveDate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for MyNaiveDate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
