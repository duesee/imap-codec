use crate::codec::Codec;
use chrono::{DateTime, FixedOffset, NaiveDate};

impl Codec for DateTime<FixedOffset> {
    fn serialize(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z")).into_bytes()
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl Codec for NaiveDate {
    fn serialize(&self) -> Vec<u8> {
        format!("\"{}\"", self.format("%d-%b-%Y")).into_bytes()
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
