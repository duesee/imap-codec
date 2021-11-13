use std::{
    convert::{TryFrom, TryInto},
    io::Write,
};

#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::{codec::Encode, parse::sequence::sequence_set, utils::join_serializable};

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceSet(pub(crate) Vec<Sequence>);

impl Encode for SequenceSet {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        join_serializable(&self.0, b",", writer)
    }
}

impl<'a> SequenceSet {
    pub fn iter(&'a self, strategy: Strategy) -> impl Iterator<Item = u32> + 'a {
        match strategy {
            Strategy::Naive { largest } => SequenceSetIterNaive {
                iter: self.0.iter(),
                active_range: None,
                largest,
            },
        }
    }
}

pub enum Strategy {
    Naive { largest: u32 },
}

pub struct SequenceSetIterNaive<'a> {
    iter: core::slice::Iter<'a, Sequence>,
    active_range: Option<std::ops::RangeInclusive<u32>>,
    largest: u32,
}

impl<'a> Iterator for SequenceSetIterNaive<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut range) = self.active_range {
                if let Some(seq_or_uid) = range.next() {
                    return Some(seq_or_uid);
                } else {
                    self.active_range = None;
                }
            }

            match self.iter.next() {
                Some(seq) => match seq {
                    Sequence::Single(seq_no) => {
                        return Some(seq_no.expand(self.largest));
                    }
                    Sequence::Range(from, to) => {
                        let from = from.expand(self.largest);
                        let to = to.expand(self.largest);
                        self.active_range = Some(from..=to);
                    }
                },
                None => return None,
            }
        }
    }
}

impl Encode for Sequence {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Sequence::Single(seq_no) => seq_no.encode(writer),
            Sequence::Range(from, to) => {
                from.encode(writer)?;
                writer.write_all(b":")?;
                to.encode(writer)
            }
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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

impl Encode for SeqNo {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            SeqNo::Value(number) => write!(writer, "{}", number),
            SeqNo::Largest => writer.write_all(b"*"),
        }
    }
}

impl TryFrom<&str> for SequenceSet {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // FIXME: turn incomplete parser to complete?
        let blocker = format!("{}|", value);

        if let Ok((b"|", sequence)) = sequence_set(blocker.as_bytes()) {
            Ok(sequence)
        } else {
            Err(())
        }
    }
}

impl TryFrom<String> for SequenceSet {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use super::{SeqNo, Sequence, Strategy};
    use crate::{codec::Encode, types::sequence::SequenceSet};

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
            test.encode(&mut out).unwrap();
            assert_eq!(*expected, out);
        }
    }

    #[test]
    fn test_to_sequence() {
        let tests = &[
            ("1", SequenceSet(vec![Sequence::Single(SeqNo::Value(1))])),
            (
                "1,2,3",
                SequenceSet(vec![
                    Sequence::Single(SeqNo::Value(1)),
                    Sequence::Single(SeqNo::Value(2)),
                    Sequence::Single(SeqNo::Value(3)),
                ]),
            ),
            ("*", SequenceSet(vec![Sequence::Single(SeqNo::Largest)])),
            (
                "1:2",
                SequenceSet(vec![Sequence::Range(SeqNo::Value(1), SeqNo::Value(2))]),
            ),
            (
                "1:2,3",
                SequenceSet(vec![
                    Sequence::Range(SeqNo::Value(1), SeqNo::Value(2)),
                    Sequence::Single(SeqNo::Value(3)),
                ]),
            ),
            (
                "1:2,3,*",
                SequenceSet(vec![
                    Sequence::Range(SeqNo::Value(1), SeqNo::Value(2)),
                    Sequence::Single(SeqNo::Value(3)),
                    Sequence::Single(SeqNo::Largest),
                ]),
            ),
        ];

        for (test, expected) in tests.into_iter() {
            let got = SequenceSet::try_from(*test).unwrap();
            assert_eq!(*expected, got);
        }
    }

    #[test]
    fn test_sequence_set_iter() {
        let tests = &[
            ("*", vec![3]),
            ("1:*", vec![1, 2, 3]),
            ("5,1:*,2:*", vec![5, 1, 2, 3, 2, 3]),
            ("*:2", vec![]),
            ("*:*", vec![3]),
            ("4:6,*", vec![4, 5, 6, 3]),
        ];

        for (test, expected) in tests.into_iter() {
            let seq_set = SequenceSet::try_from(*test).unwrap();
            let got: Vec<u32> = seq_set.iter(Strategy::Naive { largest: 3 }).collect();
            assert_eq!(*expected, got);
        }
    }
}
