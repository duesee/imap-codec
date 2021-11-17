use std::{
    fmt::{Debug, Display, Formatter},
    io::Write,
};

use chrono::{DateTime, FixedOffset, NaiveDate};
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::codec::Encode;

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

impl Encode for MyDateTime {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
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

impl Encode for MyNaiveDate {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

impl Encode for DateTime<FixedOffset> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z"))
    }
}

impl Encode for NaiveDate {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y"))
    }
}
