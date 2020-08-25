use crate::codec::Codec;
use chrono::{DateTime, FixedOffset};

impl Codec for DateTime<FixedOffset> {
    fn serialize(&self) -> Vec<u8> {
        // "DQUOTE date-day-fixed "-" date-month "-" date-year SP time SP zone DQUOTE"
        format!("\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z")).into_bytes()
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
