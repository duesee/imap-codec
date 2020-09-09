use crate::codec::Codec;
use chrono::{DateTime, FixedOffset, NaiveDate};

impl Codec for DateTime<FixedOffset> {
    fn serialize(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z")).into_bytes()
    }
}

impl Codec for NaiveDate {
    fn serialize(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y")).into_bytes()
    }
}
