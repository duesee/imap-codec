use std::{
    num::{NonZeroU32, ParseIntError, TryFromIntError},
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    str::FromStr,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::NonEmptyVec;

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceSet(pub NonEmptyVec<Sequence>);

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sequence {
    Single(SeqOrUid),
    Range(SeqOrUid, SeqOrUid),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SeqOrUid {
    Value(NonZeroU32),
    Asterisk,
}

macro_rules! impl_try_from_for_seq_or_uid {
    ($num:ty) => {
        impl TryFrom<$num> for SeqOrUid {
            type Error = TryFromIntError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                Ok(Self::Value(NonZeroU32::try_from(u32::try_from(value)?)?))
            }
        }
    };
}

impl_try_from_for_seq_or_uid!(i8);
impl_try_from_for_seq_or_uid!(i16);
impl_try_from_for_seq_or_uid!(i32);
impl_try_from_for_seq_or_uid!(i64);
impl_try_from_for_seq_or_uid!(isize);
impl_try_from_for_seq_or_uid!(u8);
impl_try_from_for_seq_or_uid!(u16);
impl_try_from_for_seq_or_uid!(u32);
impl_try_from_for_seq_or_uid!(u64);
impl_try_from_for_seq_or_uid!(usize);

impl<'a> SequenceSet {
    pub fn iter(&'a self, strategy: Strategy) -> impl Iterator<Item = NonZeroU32> + 'a {
        match strategy {
            Strategy::Naive { largest } => SequenceSetIterNaive {
                iter: self.0.as_ref().iter(),
                active_range: None,
                largest,
            },
        }
    }
}

impl TryFrom<&str> for SequenceSet {
    type Error = SequenceSetError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut results = vec![];

        for seq in value.split(',') {
            results.push(Sequence::try_from(seq)?);
        }

        Ok(SequenceSet(
            NonEmptyVec::try_from(results).map_err(|_| SequenceSetError::Empty)?,
        ))
    }
}

impl TryFrom<String> for SequenceSet {
    type Error = SequenceSetError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<u32> for SequenceSet {
    type Error = SequenceSetError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let value = NonZeroU32::try_from(value).map_err(|_| SequenceSetError::Zero)?;

        Ok(Self(NonEmptyVec::from(Sequence::Single(SeqOrUid::Value(
            value,
        )))))
    }
}

macro_rules! impl_try_from_num_slice_for_sequence_set {
    ($num:ty) => {
        impl TryFrom<&[$num]> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(value: &[$num]) -> Result<Self, Self::Error> {
                let mut vec = Vec::with_capacity(value.len());

                for value in value {
                    vec.push(Sequence::Single(SeqOrUid::try_from(*value).map_err(
                        |_| SequenceSetError::Sequence(SequenceError::Invalid),
                    )?));
                }

                Ok(Self(
                    NonEmptyVec::try_from(vec).map_err(|_| SequenceSetError::Empty)?,
                ))
            }
        }
    };
}

impl_try_from_num_slice_for_sequence_set!(i8);
impl_try_from_num_slice_for_sequence_set!(i16);
impl_try_from_num_slice_for_sequence_set!(i32);
impl_try_from_num_slice_for_sequence_set!(i64);
impl_try_from_num_slice_for_sequence_set!(isize);
impl_try_from_num_slice_for_sequence_set!(u8);
impl_try_from_num_slice_for_sequence_set!(u16);
impl_try_from_num_slice_for_sequence_set!(u32);
impl_try_from_num_slice_for_sequence_set!(u64);
impl_try_from_num_slice_for_sequence_set!(usize);

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SequenceSetError {
    #[error("Must not be empty.")]
    Empty,
    #[error("Must not be zero.")]
    Zero,
    #[error("Sequence: {0}")]
    Sequence(#[from] SequenceError),
}

impl TryFrom<&str> for Sequence {
    type Error = SequenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split(':').count() {
            0 => Err(SequenceError::Empty),
            1 => Ok(Sequence::Single(SeqOrUid::try_from(value)?)),
            2 => {
                let mut split = value.split(':');

                let start = split.next().unwrap();
                let end = split.next().unwrap();

                Ok(Sequence::Range(
                    SeqOrUid::try_from(start)?,
                    SeqOrUid::try_from(end)?,
                ))
            }
            _ => Err(SequenceError::Invalid),
        }
    }
}

impl From<RangeFull> for Sequence {
    fn from(_: RangeFull) -> Self {
        Sequence::Range(
            SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
            SeqOrUid::Asterisk,
        )
    }
}

impl TryFrom<RangeFrom<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeFrom<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::try_from(range.start)?;

        Ok(Sequence::Range(SeqOrUid::Value(start), SeqOrUid::Asterisk))
    }
}

impl From<RangeFrom<NonZeroU32>> for SequenceSet {
    fn from(range: RangeFrom<NonZeroU32>) -> Self {
        SequenceSet(NonEmptyVec::from(Sequence::Range(
            SeqOrUid::Value(range.start),
            SeqOrUid::Asterisk,
        )))
    }
}

impl From<RangeFrom<NonZeroU32>> for Sequence {
    fn from(range: RangeFrom<NonZeroU32>) -> Self {
        Sequence::Range(SeqOrUid::Value(range.start), SeqOrUid::Asterisk)
    }
}

impl TryFrom<Range<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: Range<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::try_from(range.start)?;
        let end = NonZeroU32::try_from(range.end.saturating_sub(1))?;

        Ok(Sequence::Range(
            SeqOrUid::Value(start),
            SeqOrUid::Value(end),
        ))
    }
}

impl TryFrom<RangeInclusive<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeInclusive<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::try_from(*range.start())?;
        let end = NonZeroU32::try_from(*range.end())?;

        Ok(Sequence::Range(
            SeqOrUid::Value(start),
            SeqOrUid::Value(end),
        ))
    }
}

impl TryFrom<RangeTo<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeTo<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::new(1).unwrap();
        let end = NonZeroU32::try_from(range.end.saturating_sub(1))?;

        Ok(Sequence::Range(
            SeqOrUid::Value(start),
            SeqOrUid::Value(end),
        ))
    }
}

impl TryFrom<RangeToInclusive<u32>> for Sequence {
    type Error = TryFromIntError;

    fn try_from(range: RangeToInclusive<u32>) -> Result<Sequence, Self::Error> {
        let start = NonZeroU32::new(1).unwrap();
        let end = NonZeroU32::try_from(range.end)?;

        Ok(Sequence::Range(
            SeqOrUid::Value(start),
            SeqOrUid::Value(end),
        ))
    }
}

impl From<Sequence> for SequenceSet {
    fn from(seq: Sequence) -> Self {
        SequenceSet(NonEmptyVec::from(seq))
    }
}

// TODO(cleanup): Make this work and delete the code above?
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

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SequenceError {
    #[error("Sequence must not be empty.")]
    Empty,
    #[error("Invalid sequence.")]
    Invalid,
    #[error("SeqNo: {0}")]
    SeqNo(#[from] SeqNoError),
}

impl SeqOrUid {
    pub fn expand(&self, largest: NonZeroU32) -> NonZeroU32 {
        match self {
            SeqOrUid::Value(value) => *value,
            SeqOrUid::Asterisk => largest,
        }
    }
}

impl TryFrom<&str> for SeqOrUid {
    type Error = SeqNoError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "*" {
            Ok(SeqOrUid::Asterisk)
        } else {
            // This is to align parsing here with the IMAP grammar:
            // Rust's `parse::<NonZeroU32>` function accepts numbers that start with 0.
            // For example, 00001, is interpreted as 1. But this is not allowed in IMAP.
            if value.starts_with('0') {
                Err(SeqNoError::LeadingZero)
            } else {
                Ok(SeqOrUid::Value(NonZeroU32::from_str(value)?))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SeqNoError {
    #[error("Must not start with \"0\".")]
    LeadingZero,
    #[error("Parse: {0}")]
    Parse(#[from] ParseIntError),
}

// -------------------------------------------------------------------------------------------------

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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::core::NonEmptyVec;

    #[test]
    fn test_creation_of_sequence_from_u32() {
        assert_eq!(
            SequenceSet::try_from(1),
            Ok(SequenceSet(NonEmptyVec::from(Sequence::Single(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap())
            ))))
        );
        assert_eq!(SequenceSet::try_from(0), Err(SequenceSetError::Zero));
    }

    #[test]
    fn test_creation_of_sequence_from_range() {
        // 1:*
        let range = ..;
        let seq = Sequence::from(range);
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
                SeqOrUid::Asterisk
            )
        );

        // 1:*
        let range = 1..;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
                SeqOrUid::Asterisk
            )
        );

        // 1337:*
        let range = 1337..;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1337).unwrap()),
                SeqOrUid::Asterisk
            )
        );

        // 1:1336
        let range = 1..1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
                SeqOrUid::Value(NonZeroU32::new(1336).unwrap())
            )
        );

        // 1:1337
        let range = 1..=1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
                SeqOrUid::Value(NonZeroU32::new(1337).unwrap())
            )
        );

        // 1:1336
        let range = ..1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
                SeqOrUid::Value(NonZeroU32::new(1336).unwrap())
            )
        );

        // 1:1337
        let range = ..=1337;
        let seq = Sequence::try_from(range).unwrap();
        assert_eq!(
            seq,
            Sequence::Range(
                SeqOrUid::Value(NonZeroU32::new(1).unwrap()),
                SeqOrUid::Value(NonZeroU32::new(1337).unwrap())
            )
        );
    }

    #[test]
    fn test_creation_of_sequence_set_from_str_positive() {
        let tests = &[
            (
                "1",
                SequenceSet(
                    vec![Sequence::Single(SeqOrUid::Value(1.try_into().unwrap()))]
                        .try_into()
                        .unwrap(),
                ),
            ),
            (
                "1,2,3",
                SequenceSet(
                    vec![
                        Sequence::Single(SeqOrUid::Value(1.try_into().unwrap())),
                        Sequence::Single(SeqOrUid::Value(2.try_into().unwrap())),
                        Sequence::Single(SeqOrUid::Value(3.try_into().unwrap())),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            ),
            (
                "*",
                SequenceSet(
                    vec![Sequence::Single(SeqOrUid::Asterisk)]
                        .try_into()
                        .unwrap(),
                ),
            ),
            (
                "1:2",
                SequenceSet(
                    vec![Sequence::Range(
                        SeqOrUid::Value(1.try_into().unwrap()),
                        SeqOrUid::Value(2.try_into().unwrap()),
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
                            SeqOrUid::Value(1.try_into().unwrap()),
                            SeqOrUid::Value(2.try_into().unwrap()),
                        ),
                        Sequence::Single(SeqOrUid::Value(3.try_into().unwrap())),
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
                            SeqOrUid::Value(1.try_into().unwrap()),
                            SeqOrUid::Value(2.try_into().unwrap()),
                        ),
                        Sequence::Single(SeqOrUid::Value(3.try_into().unwrap())),
                        Sequence::Single(SeqOrUid::Asterisk),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            ),
        ];

        for (test, expected) in tests.iter() {
            let got = SequenceSet::try_from(*test).unwrap();
            assert_eq!(*expected, got);
        }
    }

    #[test]
    fn test_creation_of_sequence_set_from_str_negative() {
        let tests = &[
            "", "* ", " *", " * ", "1 ", " 1", " 1 ", "01", " 01", "01 ", " 01 ", "*1", ":", ":*",
            "*:", "*: ", "1:2:3",
        ];

        for test in tests {
            let got = SequenceSet::try_from(*test);
            print!("\"{}\" | {:?} | ", test, got.clone().unwrap_err());
            println!("{}", got.unwrap_err());
        }
    }

    #[test]
    fn test_iteration_over_some_sequence_sets() {
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
