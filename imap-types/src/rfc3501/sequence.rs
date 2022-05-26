use std::{
    convert::{TryFrom, TryInto},
    num::{NonZeroU32, TryFromIntError},
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::core::NonEmptyVec;

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceSet(pub NonEmptyVec<Sequence>);

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

impl TryFrom<&str> for SeqNo {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "*" {
            Ok(SeqNo::Largest)
        } else {
            // This is to align parsing here with the IMAP grammar:
            // Rust's `parse::<NonZeroU32>` function accepts numbers that start with 0.
            // For example, 00001, is interpreted as 1. But this is not allowed in IMAP.
            if value.starts_with('0') {
                Err(())
            } else {
                Ok(SeqNo::Value(value.parse::<NonZeroU32>().map_err(|_| ())?))
            }
        }
    }
}

impl TryFrom<&str> for Sequence {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split(':').count() {
            0 => Err(()),
            1 => Ok(Sequence::Single(SeqNo::try_from(value)?)),
            2 => {
                let mut split = value.split(':');

                let start = split.next().unwrap();
                let end = split.next().unwrap();

                Ok(Sequence::Range(
                    SeqNo::try_from(start)?,
                    SeqNo::try_from(end)?,
                ))
            }
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for SequenceSet {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut results = vec![];

        for seq in value.split(',') {
            results.push(Sequence::try_from(seq)?);
        }

        Ok(SequenceSet(NonEmptyVec::try_from(results)?))
    }
}

// TODO: Used for Arbitrary
impl TryFrom<String> for SequenceSet {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl From<RangeFull> for Sequence {
    fn from(_: RangeFull) -> Self {
        Sequence::Range(SeqNo::Value(NonZeroU32::new(1).unwrap()), SeqNo::Largest)
    }
}

impl TryFrom<RangeFrom<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeFrom<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::try_from(range.start)?;

        Ok(Sequence::Range(SeqNo::Value(start), SeqNo::Largest))
    }
}

impl TryFrom<Range<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: Range<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::try_from(range.start)?;
        let end = NonZeroU32::try_from(range.end.saturating_sub(1))?;

        Ok(Sequence::Range(SeqNo::Value(start), SeqNo::Value(end)))
    }
}

impl TryFrom<RangeInclusive<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeInclusive<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::try_from(*range.start())?;
        let end = NonZeroU32::try_from(*range.end())?;

        Ok(Sequence::Range(SeqNo::Value(start), SeqNo::Value(end)))
    }
}

impl TryFrom<RangeTo<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeTo<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::new(1).unwrap();
        let end = NonZeroU32::try_from(range.end.saturating_sub(1))?;

        Ok(Sequence::Range(SeqNo::Value(start), SeqNo::Value(end)))
    }
}

impl TryFrom<RangeToInclusive<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeToInclusive<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::new(1).unwrap();
        let end = NonZeroU32::try_from(range.end)?;

        Ok(Sequence::Range(SeqNo::Value(start), SeqNo::Value(end)))
    }
}

impl From<Sequence> for SequenceSet {
    fn from(seq: Sequence) -> Self {
        SequenceSet(unsafe { NonEmptyVec::new_unchecked(vec![seq]) })
    }
}

// TODO: Make this work and delete the code above?
//
// error[E0119]: conflicting implementations of trait `std::convert::TryFrom<_>` for type `rfc3501::sequence::Sequence`
//
// impl<R> TryFrom<R> for Sequence where R: RangeBounds<u32> {
//     type Error = TryFromIntError;
//
//     fn try_from(value: R) -> Result<Self, Self::Error> {
//         let start = match value.start_bound() {
//             Bound::Unbounded => SeqNo::Value(NonZeroU32::new(1).unwrap()),
//             Bound::Excluded(start) => SeqNo::Value(NonZeroU32::try_from(start.saturating_sub(1))?),
//             Bound::Included(start) => SeqNo::Value(NonZeroU32::try_from(start)?),
//         };
//
//         let end = match value.end_bound() {
//             Bound::Unbounded => SeqNo::Largest,
//             Bound::Excluded(end) => SeqNo::Value(NonZeroU32::try_from(end.saturating_sub(1))?),
//             Bound::Included(end) => SeqNo::Value(NonZeroU32::try_from(end)?),
//         };
//
//         Ok(Sequence::Range(start, end))
//     }
// }

#[cfg(test)]
mod test {
    use std::{
        convert::{TryFrom, TryInto},
        num::NonZeroU32,
    };

    use super::{SeqNo, Sequence, Strategy};
    use crate::{codec::Encode, sequence::SequenceSet};

    #[test]
    fn creation_of_sequence_from_range() {
        // 1:*
        let range = ..;
        let seq = Sequence::from(range);
        assert_eq!(
            seq,
            Sequence::Range(SeqNo::Value(NonZeroU32::new(1).unwrap()), SeqNo::Largest)
        );

        // 1:*
        let range = 1..;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(SeqNo::Value(NonZeroU32::new(1).unwrap()), SeqNo::Largest)
        );

        // 1337:*
        let range = 1337..;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(SeqNo::Value(NonZeroU32::new(1337).unwrap()), SeqNo::Largest)
        );

        // 1:1336
        let range = 1..1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqNo::Value(NonZeroU32::new(1).unwrap()),
                SeqNo::Value(NonZeroU32::new(1336).unwrap())
            )
        );

        // 1:1337
        let range = 1..=1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqNo::Value(NonZeroU32::new(1).unwrap()),
                SeqNo::Value(NonZeroU32::new(1337).unwrap())
            )
        );

        // 1:1336
        let range = ..1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqNo::Value(NonZeroU32::new(1).unwrap()),
                SeqNo::Value(NonZeroU32::new(1336).unwrap())
            )
        );

        // 1:1337
        let range = ..=1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqNo::Value(NonZeroU32::new(1).unwrap()),
                SeqNo::Value(NonZeroU32::new(1337).unwrap())
            )
        );
    }

    #[test]
    fn creation_of_sequence_set_from_str_positive() {
        let tests = &[
            (
                "1",
                SequenceSet(
                    vec![Sequence::Single(SeqNo::Value(1.try_into().unwrap()))]
                        .try_into()
                        .unwrap(),
                ),
            ),
            (
                "1,2,3",
                SequenceSet(
                    vec![
                        Sequence::Single(SeqNo::Value(1.try_into().unwrap())),
                        Sequence::Single(SeqNo::Value(2.try_into().unwrap())),
                        Sequence::Single(SeqNo::Value(3.try_into().unwrap())),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            ),
            (
                "*",
                SequenceSet(vec![Sequence::Single(SeqNo::Largest)].try_into().unwrap()),
            ),
            (
                "1:2",
                SequenceSet(
                    vec![Sequence::Range(
                        SeqNo::Value(1.try_into().unwrap()),
                        SeqNo::Value(2.try_into().unwrap()),
                    )]
                    .try_into()
                    .unwrap(),
                ),
            ),
            (
                "1:2,3",
                SequenceSet(
                    vec![
                        Sequence::Range(
                            SeqNo::Value(1.try_into().unwrap()),
                            SeqNo::Value(2.try_into().unwrap()),
                        ),
                        Sequence::Single(SeqNo::Value(3.try_into().unwrap())),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            ),
            (
                "1:2,3,*",
                SequenceSet(
                    vec![
                        Sequence::Range(
                            SeqNo::Value(1.try_into().unwrap()),
                            SeqNo::Value(2.try_into().unwrap()),
                        ),
                        Sequence::Single(SeqNo::Value(3.try_into().unwrap())),
                        Sequence::Single(SeqNo::Largest),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            ),
        ];

        for (test, expected) in tests.into_iter() {
            let got = SequenceSet::try_from(*test).unwrap();
            assert_eq!(*expected, got);
        }
    }

    #[test]
    fn creation_of_sequence_set_from_str_negative() {
        let tests = &[
            "", "* ", " *", " * ", "1 ", " 1", " 1 ", "01", " 01", "01 ", " 01 ", "*1", ":", ":*",
            "*:", "*: ",
        ];

        for test in tests {
            let got = SequenceSet::try_from(*test);
            assert_eq!(Err(()), got);
        }
    }

    #[test]
    fn serialization_of_some_sequence_sets() {
        let tests = [
            (
                Sequence::Single(SeqNo::Value(1.try_into().unwrap())),
                b"1".as_ref(),
            ),
            (Sequence::Single(SeqNo::Largest), b"*".as_ref()),
            (
                Sequence::Range(SeqNo::Value(1.try_into().unwrap()), SeqNo::Largest),
                b"1:*".as_ref(),
            ),
        ];

        for (test, expected) in tests {
            let mut out = Vec::new();
            test.encode(&mut out).unwrap();
            assert_eq!(*expected, out);
        }
    }

    #[test]
    fn iteration_over_some_sequence_sets() {
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
