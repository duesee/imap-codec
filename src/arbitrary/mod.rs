use std::convert::TryFrom;

use arbitrary::{Arbitrary, Unstructured};
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

use crate::types::{
    command::SearchKey,
    core::{AString, Atom, Literal, NonEmptyVec, Quoted, Tag, Text},
    datetime::{MyDateTime, MyNaiveDate},
    mailbox::{ListCharString, Mailbox},
    sequence::SequenceSet,
    AuthMechanismOther,
};

macro_rules! implement_tryfrom {
    ($target:ty, $from:ty) => {
        impl<'a> Arbitrary<'a> for $target {
            fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                match <$target>::try_from(<$from>::arbitrary(u)?) {
                    Ok(passed) => Ok(passed),
                    Err(_) => Err(arbitrary::Error::IncorrectFormat),
                }
            }
        }
    };
}

macro_rules! implement_tryfrom_t {
    ($target:ty, $from:ty) => {
        impl<'a, T> Arbitrary<'a> for $target
        where
            T: Arbitrary<'a>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                match <$target>::try_from(<$from>::arbitrary(u)?) {
                    Ok(passed) => Ok(passed),
                    Err(_) => Err(arbitrary::Error::IncorrectFormat),
                }
            }
        }
    };
}

implement_tryfrom! { Atom, String }
implement_tryfrom! { Quoted, String }
implement_tryfrom! { Tag, String }
implement_tryfrom! { Text, String }
implement_tryfrom! { ListCharString, String }
implement_tryfrom! { Mailbox, String }
implement_tryfrom! { AuthMechanismOther, String }
implement_tryfrom! { SequenceSet, String }
implement_tryfrom! { Literal, Vec<u8> }
implement_tryfrom_t! { NonEmptyVec<T>, Vec<T> }

impl<'a> Arbitrary<'a> for SearchKey {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        use SearchKey::*;

        use crate::types::sequence::SequenceSet as SequenceSetData;

        let search_keys = &[
            // And(NonEmptyVec::<SearchKey>::arbitrary(u)?), // TODO
            SequenceSet(SequenceSetData::arbitrary(u)?),
            All,
            Answered,
            Bcc(AString::arbitrary(u)?),
            Before(MyNaiveDate::arbitrary(u)?),
            Body(AString::arbitrary(u)?),
            Cc(AString::arbitrary(u)?),
            Deleted,
            Draft,
            Flagged,
            From(AString::arbitrary(u)?),
            Header(AString::arbitrary(u)?, AString::arbitrary(u)?),
            Keyword(Atom::arbitrary(u)?),
            Larger(u32::arbitrary(u)?),
            New,
            // Not(Box::<SearchKey>::arbitrary(u)?), // TODO
            Old,
            On(MyNaiveDate::arbitrary(u)?),
            // Or(Box::<SearchKey>::arbitrary(u)?, Box::<SearchKey>::arbitrary(u)?), // TODO
            Recent,
            Seen,
            SentBefore(MyNaiveDate::arbitrary(u)?),
            SentOn(MyNaiveDate::arbitrary(u)?),
            SentSince(MyNaiveDate::arbitrary(u)?),
            Since(MyNaiveDate::arbitrary(u)?),
            Smaller(u32::arbitrary(u)?),
            Subject(AString::arbitrary(u)?),
            Text(AString::arbitrary(u)?),
            To(AString::arbitrary(u)?),
            Uid(SequenceSetData::arbitrary(u)?),
            Unanswered,
            Undeleted,
            Undraft,
            Unflagged,
            Unkeyword(Atom::arbitrary(u)?),
            Unseen,
        ];

        Ok(u.choose(search_keys)?.clone())
    }
}

impl<'a> Arbitrary<'a> for MyDateTime {
    fn arbitrary(_: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // TODO: make arbitrary :-)

        let local_datetime = NaiveDateTime::new(
            NaiveDate::from_ymd(1985, 2, 1),
            NaiveTime::from_hms(12, 34, 56),
        );

        Ok(MyDateTime(
            FixedOffset::east(3600)
                .from_local_datetime(&local_datetime)
                .unwrap(),
        ))
    }
}

impl<'a> Arbitrary<'a> for MyNaiveDate {
    fn arbitrary(_: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // TODO: make arbitrary!

        Ok(MyNaiveDate(NaiveDate::from_ymd_opt(2020, 2, 1).unwrap()))
    }
}
