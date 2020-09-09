use crate::codec::Encoder;
use chrono::{DateTime, FixedOffset, NaiveDate};

impl Encoder for DateTime<FixedOffset> {
    fn encode(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z")).into_bytes()
    }
}

impl Encoder for NaiveDate {
    fn encode(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y")).into_bytes()
    }
}
