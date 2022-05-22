use std::{
    convert::{TryFrom, TryInto},
    num::NonZeroU32,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::parse::sequence::sequence_set;

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceSet(pub(crate) Vec<Sequence>);

impl<'a> SequenceSet {
    pub fn iter(&'a self, strategy: Strategy) -> impl Iterator<Item = NonZeroU32> + 'a {
        match strategy {
            Strategy::Naive { largest } => SequenceSetIterNaive {
                iter: self.0.iter(),
                active_range: None,
                largest,
            },
        }
    }
}

#[derive(Debug)]
pub enum Strategy {
    Naive { largest: NonZeroU32 },
}

#[derive(Debug)]
pub struct SequenceSetIterNaive<'a> {
    iter: core::slice::Iter<'a, Sequence>,
    active_range: Option<std::ops::RangeInclusive<u32>>,
    largest: NonZeroU32,
}

impl<'a> Iterator for SequenceSetIterNaive<'a> {
    type Item = NonZeroU32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut range) = self.active_range {
                if let Some(seq_or_uid) = range.next() {
                    return Some(NonZeroU32::try_from(seq_or_uid).unwrap());
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
                        self.active_range = Some(u32::from(from)..=u32::from(to));
                    }
                },
                None => return None,
            }
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SeqNo {
    Value(NonZeroU32),
    Largest,
}

impl SeqNo {
    pub fn expand(&self, largest: NonZeroU32) -> NonZeroU32 {
        match self {
            SeqNo::Value(value) => *value,
            SeqNo::Largest => largest,
        }
    }
}

impl TryFrom<&str> for SequenceSet {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO: turn incomplete parser to complete?
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
    use std::{
        convert::{TryFrom, TryInto},
        num::NonZeroU32,
    };

    use super::{SeqNo, Sequence, Strategy};
    use crate::{codec::Encode, types::sequence::SequenceSet};

    #[test]
    fn test_sequence_serialize() {
        let tests = [
            (
                b"1".as_ref(),
                Sequence::Single(SeqNo::Value(1.try_into().unwrap())),
            ),
            (b"*".as_ref(), Sequence::Single(SeqNo::Largest)),
            (
                b"1:*".as_ref(),
                Sequence::Range(SeqNo::Value(1.try_into().unwrap()), SeqNo::Largest),
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
            (
                "1",
                SequenceSet(vec![Sequence::Single(SeqNo::Value(1.try_into().unwrap()))]),
            ),
            (
                "1,2,3",
                SequenceSet(vec![
                    Sequence::Single(SeqNo::Value(1.try_into().unwrap())),
                    Sequence::Single(SeqNo::Value(2.try_into().unwrap())),
                    Sequence::Single(SeqNo::Value(3.try_into().unwrap())),
                ]),
            ),
            ("*", SequenceSet(vec![Sequence::Single(SeqNo::Largest)])),
            (
                "1:2",
                SequenceSet(vec![Sequence::Range(
                    SeqNo::Value(1.try_into().unwrap()),
                    SeqNo::Value(2.try_into().unwrap()),
                )]),
            ),
            (
                "1:2,3",
                SequenceSet(vec![
                    Sequence::Range(
                        SeqNo::Value(1.try_into().unwrap()),
                        SeqNo::Value(2.try_into().unwrap()),
                    ),
                    Sequence::Single(SeqNo::Value(3.try_into().unwrap())),
                ]),
            ),
            (
                "1:2,3,*",
                SequenceSet(vec![
                    Sequence::Range(
                        SeqNo::Value(1.try_into().unwrap()),
                        SeqNo::Value(2.try_into().unwrap()),
                    ),
                    Sequence::Single(SeqNo::Value(3.try_into().unwrap())),
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
        let tests = vec![
            ("*", vec![3]),
            ("1:*", vec![1, 2, 3]),
            ("5,1:*,2:*", vec![5, 1, 2, 3, 2, 3]),
            ("*:2", vec![]),
            ("*:*", vec![3]),
            ("4:6,*", vec![4, 5, 6, 3]),
        ]
        .into_iter()
        .map(|(raw, vec)| {
            (
                raw,
                vec.into_iter()
                    .map(|num| num.try_into().unwrap())
                    .collect::<Vec<NonZeroU32>>(),
            )
        })
        .collect::<Vec<(&str, Vec<NonZeroU32>)>>();

        for (test, expected) in tests {
            let seq_set = SequenceSet::try_from(test).unwrap();
            let got: Vec<NonZeroU32> = seq_set
                .iter(Strategy::Naive {
                    largest: 3.try_into().unwrap(),
                })
                .collect();
            assert_eq!(*expected, got);
        }
    }
}
