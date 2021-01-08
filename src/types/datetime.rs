use crate::codec::Encode;
use chrono::{DateTime, FixedOffset, NaiveDate};
use std::io::Write;

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
