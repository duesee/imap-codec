use crate::codec::Serialize;
use chrono::{DateTime, FixedOffset, NaiveDate};

impl Serialize for DateTime<FixedOffset> {
    fn serialize(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z")).into_bytes()
    }
}

impl Serialize for NaiveDate {
    fn serialize(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y")).into_bytes()
    }
}
