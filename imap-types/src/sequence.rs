use std::{
    cmp::max,
    collections::VecDeque,
    fmt::Debug,
    iter::Rev,
    num::NonZeroU32,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    str::FromStr,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::Vec1,
    error::{ValidationError, ValidationErrorKind},
};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct SequenceSet(pub Vec1<Sequence>);

impl From<Sequence> for SequenceSet {
    fn from(sequence: Sequence) -> Self {
        Self(Vec1::from(sequence))
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
            type Error = ValidationError;

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
    type Error = ValidationError;

    fn try_from(sequences: Vec<Sequence>) -> Result<Self, Self::Error> {
        Ok(Self(Vec1::try_from(sequences).map_err(|_| {
            ValidationError::new(ValidationErrorKind::Empty)
        })?))
    }
}

impl TryFrom<Vec<NonZeroU32>> for SequenceSet {
    type Error = ValidationError;

    fn try_from(sequences: Vec<NonZeroU32>) -> Result<Self, Self::Error> {
        Ok(Self(
            Vec1::try_from(
                sequences
                    .into_iter()
                    .map(Sequence::from)
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| ValidationError::new(ValidationErrorKind::Empty))?,
        ))
    }
}

impl TryFrom<&str> for SequenceSet {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for SequenceSet {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut results = vec![];

        for seq in value.split(',') {
            results.push(Sequence::try_from(seq)?);
        }

        Ok(SequenceSet(Vec1::try_from(results).map_err(|_| {
            ValidationError::new(ValidationErrorKind::Empty)
        })?))
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
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
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for Sequence {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.split(':').count() {
            0 => Err(ValidationError::new(ValidationErrorKind::Empty)),
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
            _ => Err(ValidationError::new(ValidationErrorKind::Invalid)),
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, ToStatic)]
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
            type Error = ValidationError;

            fn try_from(values: &[$num]) -> Result<Self, Self::Error> {
                let mut checked = Vec::new();

                for value in values {
                    checked.push(Sequence::try_from(*value)?);
                }

                Self::try_from(checked)
            }
        }

        impl TryFrom<$num> for SequenceSet {
            type Error = ValidationError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(value)?))
            }
        }

        impl TryFrom<$num> for Sequence {
            type Error = ValidationError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                Ok(Self::from(SeqOrUid::try_from(value)?))
            }
        }

        impl TryFrom<$num> for SeqOrUid {
            type Error = ValidationError;

            fn try_from(value: $num) -> Result<Self, Self::Error> {
                if let Ok(value) = u32::try_from(value) {
                    if let Ok(value) = NonZeroU32::try_from(value) {
                        return Ok(Self::Value(value));
                    }
                }

                Err(ValidationError::new(ValidationErrorKind::Invalid))
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
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for SeqOrUid {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "*" {
            Ok(SeqOrUid::Asterisk)
        } else {
            // This is to align parsing here with the IMAP grammar:
            // Rust's `parse::<NonZeroU32>` function accepts numbers that start with 0.
            // For example, 00001, is interpreted as 1. But this is not allowed in IMAP.
            if value.starts_with('0') {
                Err(ValidationError::new(ValidationErrorKind::Invalid))
            } else {
                Ok(SeqOrUid::Value(NonZeroU32::from_str(value).map_err(
                    |_| ValidationError::new(ValidationErrorKind::Invalid),
                )?))
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

macro_rules! impl_try_from_num_range {
    ($num:ty) => {
        impl TryFrom<RangeFrom<$num>> for SequenceSet {
            type Error = ValidationError;

            fn try_from(range: RangeFrom<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<RangeTo<$num>> for SequenceSet {
            type Error = ValidationError;

            fn try_from(range: RangeTo<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<RangeToInclusive<$num>> for SequenceSet {
            type Error = ValidationError;

            fn try_from(range: RangeToInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<Range<$num>> for SequenceSet {
            type Error = ValidationError;

            fn try_from(range: Range<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        impl TryFrom<RangeInclusive<$num>> for SequenceSet {
            type Error = ValidationError;

            fn try_from(range: RangeInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::from(Sequence::try_from(range)?))
            }
        }

        // -----------------------------------------------------------------------------------------

        impl TryFrom<RangeFrom<$num>> for Sequence {
            type Error = ValidationError;

            fn try_from(range: RangeFrom<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::try_from(range.start)?,
                    SeqOrUid::Asterisk,
                ))
            }
        }

        impl TryFrom<RangeTo<$num>> for Sequence {
            type Error = ValidationError;

            fn try_from(range: RangeTo<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::from(ONE),
                    SeqOrUid::try_from(range.end.saturating_sub(1))?,
                ))
            }
        }

        impl TryFrom<RangeToInclusive<$num>> for Sequence {
            type Error = ValidationError;

            fn try_from(range: RangeToInclusive<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::from(ONE),
                    SeqOrUid::try_from(range.end)?,
                ))
            }
        }

        impl TryFrom<Range<$num>> for Sequence {
            type Error = ValidationError;

            fn try_from(range: Range<$num>) -> Result<Self, Self::Error> {
                Ok(Self::Range(
                    SeqOrUid::try_from(range.start)?,
                    SeqOrUid::try_from(range.end.saturating_sub(1))?,
                ))
            }
        }

        impl TryFrom<RangeInclusive<$num>> for Sequence {
            type Error = ValidationError;

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
    type Error = ValidationError;

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
    type Error = ValidationError;

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
    /// Iterate over a sorted, deduplicated set of sequence numbers or UIDs.
    ///
    /// # Example
    ///
    /// ```
    /// use std::num::NonZeroU32;
    ///
    /// use imap_types::sequence::SequenceSet;
    ///
    /// let seq = SequenceSet::try_from("1:5,10:3,10,10,10,1").unwrap();
    /// let largest = NonZeroU32::new(10).unwrap();
    ///
    /// assert_eq!(
    ///     seq.iter(largest).collect::<Vec<_>>(),
    ///     [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    ///         .into_iter()
    ///         .map(|e| NonZeroU32::new(e).unwrap())
    ///         .collect::<Vec<_>>()
    /// );
    /// ```
    pub fn iter(&'a self, largest: NonZeroU32) -> impl Iterator<Item = NonZeroU32> + 'a {
        let ranges = simplify(self.clone(), largest, true);
        let ranges = cleanup(ranges);

        SequenceSetIter {
            ranges,
            active_range: None,
        }
    }

    /// Iterate over the given set of sequence numbers or UIDs.
    ///
    /// Note: This method expands the sequence set keeping duplicates and ordering.
    ///
    /// # Example
    ///
    /// ```
    /// use std::num::NonZeroU32;
    ///
    /// use imap_types::sequence::SequenceSet;
    ///
    /// let seq = SequenceSet::try_from("1:3,1:3,3:1,1").unwrap();
    /// let largest = NonZeroU32::new(10).unwrap();
    ///
    /// assert_eq!(
    ///     seq.iter_naive(largest).collect::<Vec<_>>(),
    ///     [1u32, 2, 3, 1, 2, 3, 3, 2, 1, 1]
    ///         .into_iter()
    ///         .map(|e| NonZeroU32::new(e).unwrap())
    ///         .collect::<Vec<_>>()
    /// );
    /// ```
    pub fn iter_naive(&'a self, largest: NonZeroU32) -> impl Iterator<Item = NonZeroU32> + 'a {
        let ranges = simplify(self.clone(), largest, false);

        SequenceSetIter {
            ranges,
            active_range: None,
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

struct SequenceSetIter {
    ranges: VecDeque<(u32, u32)>,
    active_range: Option<Sorting>,
}

impl Iterator for SequenceSetIter {
    type Item = NonZeroU32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.active_range {
                Some(ref mut range) => match range.next() {
                    // We know here that `next >= 1`.
                    Some(next) => break Some(NonZeroU32::new(next).unwrap()),
                    None => self.active_range = None,
                },
                None => match self.ranges.pop_front() {
                    Some((a, b)) => {
                        if a <= b {
                            self.active_range = Some(Sorting::Ascending(a..=b));
                        } else {
                            self.active_range = Some(Sorting::Descending((b..=a).rev()));
                        }
                    }
                    None => break None,
                },
            }
        }
    }
}

enum Sorting {
    Ascending(RangeInclusive<u32>),
    Descending(Rev<RangeInclusive<u32>>),
}

impl Iterator for Sorting {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Sorting::Ascending(iter) => iter.next(),
            Sorting::Descending(iter) => iter.next(),
        }
    }
}

// Simplify sequence set into VecDeque<(u32, u32)>:
// * Use u32 instead of NonZeroU32 (for internal purposes)
// * Expand Single(a) to (a, a)
// * Sort Range(a, b) so that a <= b
fn simplify(sequence_set: SequenceSet, largest: NonZeroU32, sort: bool) -> VecDeque<(u32, u32)> {
    sequence_set
        .0
         .0
        .into_iter()
        .map(|seq| match seq {
            Sequence::Single(a) => (u32::from(a.expand(largest)), u32::from(a.expand(largest))),
            Sequence::Range(a, b) => {
                let a = u32::from(a.expand(largest));
                let b = u32::from(b.expand(largest));

                if sort {
                    if a <= b {
                        (a, b)
                    } else {
                        (b, a)
                    }
                } else {
                    (a, b)
                }
            }
        })
        .collect()
}

fn cleanup(remaining: VecDeque<(u32, u32)>) -> VecDeque<(u32, u32)> {
    let mut remaining = {
        let mut tmp = Vec::from(remaining);
        tmp.sort();
        VecDeque::from(tmp)
    };

    let mut stack = VecDeque::new();
    stack.push_back(remaining.pop_front().unwrap());

    for (x, y) in remaining.into_iter() {
        let last = stack.back_mut().unwrap();

        if last.0 <= x && x <= last.1.saturating_add(1) {
            last.1 = max(last.1, y);
        } else {
            stack.push_back((x, y));
        }
    }

    stack
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::core::Vec1;

    #[test]
    fn test_creation_of_sequence_from_u32() {
        assert_eq!(
            SequenceSet::try_from(1),
            Ok(SequenceSet(Vec1::from(Sequence::Single(SeqOrUid::Value(
                NonZeroU32::new(1).unwrap()
            )))))
        );
        assert_eq!(
            SequenceSet::try_from(0),
            Err(ValidationError::new(ValidationErrorKind::Invalid))
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
            ("*:2", vec![3, 2]),
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
            let got: Vec<NonZeroU32> = seq_set.iter_naive(3.try_into().unwrap()).collect();
            assert_eq!(*expected, got);
        }
    }

    /// See https://github.com/duesee/imap-codec/issues/411
    #[test]
    fn test_issue_411() {
        let seq = SequenceSet::try_from("22,21,22,*:20").unwrap();
        let largest = NonZeroU32::new(23).unwrap();

        // Naive
        {
            let expected = [22, 21, 22, 23, 22, 21, 20]
                .map(|n| NonZeroU32::new(n).unwrap())
                .to_vec();
            let got: Vec<_> = seq.iter_naive(largest).collect();

            assert_eq!(expected, got);
        }

        // Clean
        {
            let expected = [20, 21, 22, 23]
                .map(|n| NonZeroU32::new(n).unwrap())
                .to_vec();
            let got: Vec<_> = seq.iter(largest).collect();

            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_clean() {
        let tests = vec![
            "1",
            "2",
            "*",
            "1:*",
            "2:*",
            "*:*",
            "3,2,1",
            "3,2,2,2,1,1,1",
            "3:1,5:1,1:2,1:1",
            "4:5,5:1,1:2,1:1,*:*,*:10,1:100",
        ];

        for test in tests {
            let seq = SequenceSet::try_from(test).unwrap();
            let largest = NonZeroU32::new(13).unwrap();

            let naive = {
                let mut naive: Vec<_> = seq.iter_naive(largest).collect();
                naive.sort();
                naive.dedup();
                naive
            };
            let clean: Vec<_> = seq.iter(largest).collect();

            assert_eq!(naive, clean);
        }
    }
}
