use std::{
    num::NonZeroU32,
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

pub const ONE: NonZeroU32 = match NonZeroU32::new(1) {
    Some(one) => one,
    None => panic!(),
};
pub const MIN: NonZeroU32 = ONE;
pub const MAX: NonZeroU32 = match NonZeroU32::new(u32::MAX) {
    Some(max) => max,
    None => panic!(),
};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceSet(pub NonEmptyVec<Sequence>);

impl From<Sequence> for SequenceSet {
    fn from(sequence: Sequence) -> Self {
        Self(NonEmptyVec::from(sequence))
    }
}

macro_rules! impl_from_t_for_sequence_set {
    ($thing:ty) => {
        impl From<$thing> for SequenceSet {
            fn from(value: $thing) -> Self {
                Self::from(Sequence::from(value))
            }
        }
    };
}

macro_rules! impl_try_from_t_for_sequence_set {
    ($thing:ty) => {
        impl TryFrom<$thing> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(value: $thing) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(value)?))
            }
        }
    };
}

impl_from_t_for_sequence_set!(SeqOrUid);
impl_from_t_for_sequence_set!(NonZeroU32);
impl_from_t_for_sequence_set!(RangeFull);
impl_from_t_for_sequence_set!(RangeFrom<NonZeroU32>);
impl_try_from_t_for_sequence_set!(RangeTo<NonZeroU32>);
impl_from_t_for_sequence_set!(RangeToInclusive<NonZeroU32>);
impl_try_from_t_for_sequence_set!(Range<NonZeroU32>);
impl_from_t_for_sequence_set!(RangeInclusive<NonZeroU32>);

// `SequenceSet::try_from` implementations.

impl TryFrom<Vec<Sequence>> for SequenceSet {
    type Error = SequenceSetError;

    fn try_from(sequences: Vec<Sequence>) -> Result<Self, Self::Error> {
        Ok(Self(
            NonEmptyVec::try_from(sequences).map_err(|_| SequenceSetError::Empty)?,
        ))
    }
}

impl TryFrom<Vec<NonZeroU32>> for SequenceSet {
    type Error = SequenceSetError;

    fn try_from(sequences: Vec<NonZeroU32>) -> Result<Self, Self::Error> {
        Ok(Self(
            NonEmptyVec::try_from(
                sequences
                    .into_iter()
                    .map(Sequence::from)
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| SequenceSetError::Empty)?,
        ))
    }
}

impl TryFrom<&str> for SequenceSet {
    type Error = SequenceSetError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for SequenceSet {
    type Err = SequenceSetError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut results = vec![];

        for seq in value.split(',') {
            results.push(Sequence::try_from(seq)?);
        }

        Ok(SequenceSet(
            NonEmptyVec::try_from(results).map_err(|_| SequenceSetError::Empty)?,
        ))
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sequence {
    Single(SeqOrUid),
    Range(SeqOrUid, SeqOrUid),
}

impl From<SeqOrUid> for Sequence {
    fn from(value: SeqOrUid) -> Self {
        Self::Single(value)
    }
}

impl From<NonZeroU32> for Sequence {
    fn from(value: NonZeroU32) -> Self {
        Self::Single(SeqOrUid::from(value))
    }
}

impl TryFrom<&str> for Sequence {
    type Error = SequenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for Sequence {
    type Err = SequenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SeqOrUid {
    Value(NonZeroU32),
    Asterisk,
}

impl From<NonZeroU32> for SeqOrUid {
    fn from(value: NonZeroU32) -> Self {
        Self::Value(value)
    }
}

macro_rules! impl_try_from_num {
    ($num:ty) => {
        impl TryFrom<&[$num]> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(values: &[$num]) -> Result<Self, Self::Error> {
                let mut checked = Vec::new();

                for value in values {
                    checked.push(Sequence::try_from(*value)?);
                }

                Self::try_from(checked)
            }
        }

        impl TryFrom<$num> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(value)?))
            }
        }

        impl TryFrom<$num> for Sequence {
            type Error = SequenceError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                Ok(Self::from(SeqOrUid::try_from(value)?))
            }
        }

        impl TryFrom<$num> for SeqOrUid {
            type Error = SeqOrUidError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                if let Ok(value) = u32::try_from(value) {
                    if let Ok(value) = NonZeroU32::try_from(value) {
                        return Ok(Self::Value(value));
                    }
                }

                Err(SeqOrUidError::Invalid)
            }
        }
    };
}

impl_try_from_num!(i8);
impl_try_from_num!(i16);
impl_try_from_num!(i32);
impl_try_from_num!(i64);
impl_try_from_num!(isize);
impl_try_from_num!(u8);
impl_try_from_num!(u16);
impl_try_from_num!(u32);
impl_try_from_num!(u64);
impl_try_from_num!(usize);

impl TryFrom<&str> for SeqOrUid {
    type Error = SeqOrUidError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for SeqOrUid {
    type Err = SeqOrUidError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "*" {
            Ok(SeqOrUid::Asterisk)
        } else {
            // This is to align parsing here with the IMAP grammar:
            // Rust's `parse::<NonZeroU32>` function accepts numbers that start with 0.
            // For example, 00001, is interpreted as 1. But this is not allowed in IMAP.
            if value.starts_with('0') {
                Err(SeqOrUidError::LeadingZero)
            } else {
                Ok(SeqOrUid::Value(
                    NonZeroU32::from_str(value).map_err(|_| SeqOrUidError::Invalid)?,
                ))
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

macro_rules! impl_try_from_num_range {
    ($num:ty) => {
        impl TryFrom<RangeFrom<$num>> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(range: RangeFrom<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<RangeTo<$num>> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(range: RangeTo<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<RangeToInclusive<$num>> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(range: RangeToInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<Range<$num>> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(range: Range<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<RangeInclusive<$num>> for SequenceSet {
            type Error = SequenceSetError;

            fn try_from(range: RangeInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        // -----------------------------------------------------------------------------------------

        impl TryFrom<RangeFrom<$num>> for Sequence {
            type Error = SequenceError;

            fn try_from(range: RangeFrom<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::try_from(range.start)?,
                    SeqOrUid::Asterisk,
                ))
            }
        }

        impl TryFrom<RangeTo<$num>> for Sequence {
            type Error = SequenceError;

            fn try_from(range: RangeTo<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::from(ONE),
                    SeqOrUid::try_from(range.end.saturating_sub(1))?,
                ))
            }
        }

        impl TryFrom<RangeToInclusive<$num>> for Sequence {
            type Error = SequenceError;

            fn try_from(range: RangeToInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::from(ONE),
                    SeqOrUid::try_from(range.end)?,
                ))
            }
        }

        impl TryFrom<Range<$num>> for Sequence {
            type Error = SequenceError;

            fn try_from(range: Range<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::try_from(range.start)?,
                    SeqOrUid::try_from(range.end.saturating_sub(1))?,
                ))
            }
        }

        impl TryFrom<RangeInclusive<$num>> for Sequence {
            type Error = SequenceError;

            fn try_from(range: RangeInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::try_from(*range.start())?,
                    SeqOrUid::try_from(*range.end())?,
                ))
            }
        }
    };
}

impl_try_from_num_range!(i8);
impl_try_from_num_range!(i16);
impl_try_from_num_range!(i32);
impl_try_from_num_range!(i64);
impl_try_from_num_range!(isize);
impl_try_from_num_range!(u8);
impl_try_from_num_range!(u16);
impl_try_from_num_range!(u32);
impl_try_from_num_range!(u64);
impl_try_from_num_range!(usize);

impl From<RangeFull> for Sequence {
    fn from(_: RangeFull) -> Self {
        Self::from(MIN..)
    }
}

impl From<RangeFrom<NonZeroU32>> for Sequence {
    fn from(range: RangeFrom<NonZeroU32>) -> Self {
        Self::Range(SeqOrUid::from(range.start), SeqOrUid::Asterisk)
    }
}

impl TryFrom<RangeTo<NonZeroU32>> for Sequence {
    type Error = SequenceError;

    fn try_from(range: RangeTo<NonZeroU32>) -> Result<Self, Self::Error> {
        Self::try_from(MIN..range.end)
    }
}

impl From<RangeToInclusive<NonZeroU32>> for Sequence {
    fn from(range: RangeToInclusive<NonZeroU32>) -> Self {
        Self::from(MIN..=range.end)
    }
}

impl TryFrom<Range<NonZeroU32>> for Sequence {
    type Error = SequenceError;

    fn try_from(range: Range<NonZeroU32>) -> Result<Self, Self::Error> {
        Ok(Self::Range(
            SeqOrUid::from(MIN),
            SeqOrUid::try_from(range.end.get().saturating_sub(1))?,
        ))
    }
}

impl From<RangeInclusive<NonZeroU32>> for Sequence {
    fn from(range: RangeInclusive<NonZeroU32>) -> Self {
        Self::Range(SeqOrUid::from(*range.start()), SeqOrUid::from(*range.end()))
    }
}

// -------------------------------------------------------------------------------------------------

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

impl SeqOrUid {
    pub fn expand(&self, largest: NonZeroU32) -> NonZeroU32 {
        match self {
            SeqOrUid::Value(value) => *value,
            SeqOrUid::Asterisk => largest,
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug)]
#[non_exhaustive]
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

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SequenceSetError {
    #[error("Empty sequence set is not allowed")]
    Empty,
    #[error(transparent)]
    Sequence(#[from] SequenceError),
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SequenceError {
    #[error("Empty sequence is not allowed")]
    Empty,
    #[error("Invalid sequence")]
    Invalid,
    #[error(transparent)]
    SeqOrUid(#[from] SeqOrUidError),
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SeqOrUidError {
    #[error("Leading zeroes are not allowed")]
    LeadingZero,
    #[error("Out of range")]
    Invalid,
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
        assert_eq!(
            SequenceSet::try_from(0),
            Err(SequenceSetError::Sequence(SequenceError::SeqOrUid(
                SeqOrUidError::Invalid
            )))
        );
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
