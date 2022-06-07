use std::convert::TryFrom;

use arbitrary::{Arbitrary, Unstructured};
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

use crate::{
    command::SearchKey,
    core::{AString, Atom, AtomExt, Literal, NonEmptyVec, Quoted, QuotedChar, Tag, Text},
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

implement_tryfrom! { Atom<'a>, &str }
implement_tryfrom! { AtomExt<'a>, &str }
implement_tryfrom! { Quoted<'a>, &str }
implement_tryfrom! { Tag<'a>, &str }
implement_tryfrom! { Text<'a>, &str }
implement_tryfrom! { ListCharString<'a>, &str }
implement_tryfrom! { QuotedChar, char }
implement_tryfrom! { Mailbox<'a>, &str }
implement_tryfrom! { AuthMechanismOther<'a>, Atom<'a> }
implement_tryfrom! { SequenceSet, &str }
implement_tryfrom! { Literal<'a>, &[u8] }
implement_tryfrom_t! { NonEmptyVec<T>, Vec<T> }

impl<'a> Arbitrary<'a> for SearchKey<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        use SearchKey::*;

        use crate::sequence::SequenceSet as SequenceSetData;

        Ok(match u.int_in_range(0u8..=36)? {
            0 => And(NonEmptyVec::<SearchKey>::arbitrary(u)?),
            1 => SequenceSet(SequenceSetData::arbitrary(u)?),
            2 => All,
            3 => Answered,
            4 => Bcc(AString::arbitrary(u)?),
            5 => Before(MyNaiveDate::arbitrary(u)?),
            6 => Body(AString::arbitrary(u)?),
            7 => Cc(AString::arbitrary(u)?),
            8 => Deleted,
            9 => Draft,
            10 => Flagged,
            11 => From(AString::arbitrary(u)?),
            12 => Header(AString::arbitrary(u)?, AString::arbitrary(u)?),
            13 => Keyword(Atom::arbitrary(u)?),
            14 => Larger(u32::arbitrary(u)?),
            15 => New,
            16 => Not(Box::<SearchKey>::arbitrary(u)?),
            17 => Old,
            18 => On(MyNaiveDate::arbitrary(u)?),
            19 => Or(
                Box::<SearchKey>::arbitrary(u)?,
                Box::<SearchKey>::arbitrary(u)?,
            ),
            20 => Recent,
            21 => Seen,
            22 => SentBefore(MyNaiveDate::arbitrary(u)?),
            23 => SentOn(MyNaiveDate::arbitrary(u)?),
            24 => SentSince(MyNaiveDate::arbitrary(u)?),
            25 => Since(MyNaiveDate::arbitrary(u)?),
            26 => Smaller(u32::arbitrary(u)?),
            27 => Subject(AString::arbitrary(u)?),
            28 => Text(AString::arbitrary(u)?),
            29 => To(AString::arbitrary(u)?),
            30 => Uid(SequenceSetData::arbitrary(u)?),
            31 => Unanswered,
            32 => Undeleted,
            33 => Undraft,
            34 => Unflagged,
            35 => Unkeyword(Atom::arbitrary(u)?),
            36 => Unseen,
            _ => unreachable!(),
        })
    }
}

impl<'a> Arbitrary<'a> for MyDateTime {
    fn arbitrary(_: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // FIXME(#30): make arbitrary :-)

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
        // FIXME(#30): make arbitrary!

        Ok(MyNaiveDate(NaiveDate::from_ymd_opt(2020, 2, 1).unwrap()))
    }
}
