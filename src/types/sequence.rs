use crate::{codec::Codec, parse::sequence::sequence_set};

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

pub trait ToSequence {
    fn to_sequence(self) -> Result<Vec<Sequence>, ()>;
}

impl ToSequence for Sequence {
    fn to_sequence(self) -> Result<Vec<Sequence>, ()> {
        Ok(vec![self])
    }
}

impl ToSequence for Vec<Sequence> {
    fn to_sequence(self) -> Result<Vec<Sequence>, ()> {
        Ok(self)
    }
}

impl ToSequence for &str {
    fn to_sequence(self) -> Result<Vec<Sequence>, ()> {
        // FIXME: turn incomplete parser to complete?
        let blocker = format!("{}|", self);

        if let Ok((b"|", sequence)) = sequence_set(blocker.as_bytes()) {
            Ok(sequence)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::sequence::{SeqNo, Sequence, ToSequence};

    #[test]
    fn test_to_sequence() {
        let tests = [
            ("1", vec![Sequence::Single(SeqNo::Value(1))]),
            (
                "1,2,3",
                vec![
                    Sequence::Single(SeqNo::Value(1)),
                    Sequence::Single(SeqNo::Value(2)),
                    Sequence::Single(SeqNo::Value(3)),
                ],
            ),
            ("*", vec![Sequence::Single(SeqNo::Unlimited)]),
            (
                "1:2",
                vec![Sequence::Range(SeqNo::Value(1), SeqNo::Value(2))],
            ),
            (
                "1:2,3",
                vec![
                    Sequence::Range(SeqNo::Value(1), SeqNo::Value(2)),
                    Sequence::Single(SeqNo::Value(3)),
                ],
            ),
            (
                "1:2,3,*",
                vec![
                    Sequence::Range(SeqNo::Value(1), SeqNo::Value(2)),
                    Sequence::Single(SeqNo::Value(3)),
                    Sequence::Single(SeqNo::Unlimited),
                ],
            ),
        ];

        for (test, expected) in tests.iter() {
            let got = test.to_sequence().unwrap();
            assert_eq!(*expected, got);
        }
    }
}
