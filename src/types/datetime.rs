use crate::codec::Serialize;
use chrono::{DateTime, FixedOffset, NaiveDate};
use std::io::Write;

impl Serialize for DateTime<FixedOffset> {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z"))
    }
}

impl Serialize for NaiveDate {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y"))
    }
}
