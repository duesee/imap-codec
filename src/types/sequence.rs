use crate::codec::Codec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
}

impl Codec for Sequence {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Sequence::Single(seq_no) => seq_no.serialize(),
            Sequence::Range(from, to) => {
                [&from.serialize(), b":".as_ref(), &to.serialize()].concat()
            }
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqNo {
    Value(u32),
    Unlimited,
}

impl Codec for SeqNo {
    fn serialize(&self) -> Vec<u8> {
        match self {
            SeqNo::Value(number) => number.to_string().into_bytes(),
            SeqNo::Unlimited => b"*".to_vec(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
