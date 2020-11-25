use crate::{codec::Serialize, parse::sequence::sequence_set};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
}

impl Serialize for Sequence {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Sequence::Single(seq_no) => seq_no.serialize(writer),
            Sequence::Range(from, to) => {
                from.serialize(writer)?;
                writer.write_all(b":")?;
                to.serialize(writer)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqNo {
    Value(u32),
    Largest,
}

impl SeqNo {
    pub fn expand(&self, largest: u32) -> u32 {
        match self {
            SeqNo::Value(value) => *value,
            SeqNo::Largest => largest,
        }
    }
}

impl Serialize for SeqNo {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            SeqNo::Value(number) => write!(writer, "{}", number),
            SeqNo::Largest => writer.write_all(b"*"),
        }
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
    use super::{SeqNo, Sequence, ToSequence};
    use crate::codec::Serialize;

    #[test]
    fn test_sequence_serialize() {
        let tests = [
            (b"1".as_ref(), Sequence::Single(SeqNo::Value(1))),
            (b"*".as_ref(), Sequence::Single(SeqNo::Largest)), // TODO: is this a valid sequence?
            (
                b"1:*".as_ref(),
                Sequence::Range(SeqNo::Value(1), SeqNo::Largest),
            ),
        ];

        for (expected, test) in tests.iter() {
            let mut out = Vec::new();
            test.serialize(&mut out).unwrap();
            assert_eq!(*expected, out);
        }
    }

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
            ("*", vec![Sequence::Single(SeqNo::Largest)]),
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
                    Sequence::Single(SeqNo::Largest),
                ],
            ),
        ];

        for (test, expected) in tests.iter() {
            let got = test.to_sequence().unwrap();
            assert_eq!(*expected, got);
        }
    }
}
