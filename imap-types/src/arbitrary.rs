use std::convert::TryFrom;

use arbitrary::{Arbitrary, Unstructured};
use chrono::{FixedOffset, NaiveDate as ChronoNaiveDate, NaiveDateTime, NaiveTime, TimeZone};

#[cfg(feature = "ext_enable")]
use crate::extensions::enable::CapabilityEnableOther;
#[cfg(feature = "ext_quota")]
use crate::extensions::quota::ResourceOther;
use crate::{
    command::{search::SearchKey, ListCharString, SequenceSet},
    core::{AString, Atom, AtomExt, Literal, NonEmptyVec, Quoted},
    message::{AuthMechanismOther, DateTime, FlagExtension, Mailbox, MailboxOther, NaiveDate, Tag},
    response::{
        data::{Capability, QuotedChar},
        CodeOther, Text,
    },
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
implement_tryfrom! { Capability<'a>, Atom<'a> }
implement_tryfrom! { FlagExtension<'a>, Atom<'a> }
implement_tryfrom! { MailboxOther<'a>, AString<'a> }
#[cfg(feature = "ext_enable")]
implement_tryfrom! { CapabilityEnableOther<'a>, Atom<'a> }
#[cfg(feature = "ext_quota")]
implement_tryfrom! { ResourceOther<'a>, Atom<'a> }
implement_tryfrom! { AuthMechanismOther<'a>, Atom<'a> }
implement_tryfrom! { SequenceSet, &str }
implement_tryfrom_t! { NonEmptyVec<T>, Vec<T> }

impl<'a> Arbitrary<'a> for Literal<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        match Literal::try_from(<&[u8]>::arbitrary(u)?) {
            #[cfg(not(feature = "ext_literal"))]
            Ok(passed) => Ok(passed),
            #[cfg(feature = "ext_literal")]
            Ok(mut passed) => {
                passed.sync = bool::arbitrary(u)?;
                Ok(passed)
            }
            Err(_) => Err(arbitrary::Error::IncorrectFormat),
        }
    }
}

impl<'a> Arbitrary<'a> for CodeOther<'a> {
    fn arbitrary(_: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // `CodeOther` is a fallback and should usually not be created.
        Ok(CodeOther::new_unchecked(b"IMAP-CODEC-CODE-OTHER>".as_ref()))
    }
}

impl<'a> Arbitrary<'a> for SearchKey<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        fn make_search_key<'a>(u: &mut Unstructured<'a>) -> arbitrary::Result<SearchKey<'a>> {
            Ok(match u.int_in_range(0u8..=33)? {
                0 => SearchKey::SequenceSet(SequenceSet::arbitrary(u)?),
                1 => SearchKey::All,
                2 => SearchKey::Answered,
                3 => SearchKey::Bcc(AString::arbitrary(u)?),
                4 => SearchKey::Before(NaiveDate::arbitrary(u)?),
                5 => SearchKey::Body(AString::arbitrary(u)?),
                6 => SearchKey::Cc(AString::arbitrary(u)?),
                7 => SearchKey::Deleted,
                8 => SearchKey::Draft,
                9 => SearchKey::Flagged,
                10 => SearchKey::From(AString::arbitrary(u)?),
                11 => SearchKey::Header(AString::arbitrary(u)?, AString::arbitrary(u)?),
                12 => SearchKey::Keyword(Atom::arbitrary(u)?),
                13 => SearchKey::Larger(u32::arbitrary(u)?),
                14 => SearchKey::New,
                15 => SearchKey::Old,
                16 => SearchKey::On(NaiveDate::arbitrary(u)?),
                17 => SearchKey::Recent,
                18 => SearchKey::Seen,
                19 => SearchKey::SentBefore(NaiveDate::arbitrary(u)?),
                20 => SearchKey::SentOn(NaiveDate::arbitrary(u)?),
                21 => SearchKey::SentSince(NaiveDate::arbitrary(u)?),
                22 => SearchKey::Since(NaiveDate::arbitrary(u)?),
                23 => SearchKey::Smaller(u32::arbitrary(u)?),
                24 => SearchKey::Subject(AString::arbitrary(u)?),
                25 => SearchKey::Text(AString::arbitrary(u)?),
                26 => SearchKey::To(AString::arbitrary(u)?),
                27 => SearchKey::Uid(SequenceSet::arbitrary(u)?),
                28 => SearchKey::Unanswered,
                29 => SearchKey::Undeleted,
                30 => SearchKey::Undraft,
                31 => SearchKey::Unflagged,
                32 => SearchKey::Unkeyword(Atom::arbitrary(u)?),
                33 => SearchKey::Unseen,
                _ => unreachable!(),
            })
        }

        fn make_search_key_rec<'a>(
            u: &mut Unstructured<'a>,
            depth: u8,
        ) -> arbitrary::Result<SearchKey<'a>> {
            if depth == 0 {
                return make_search_key(u);
            }

            Ok(match u.int_in_range(0u8..=36)? {
                0 => SearchKey::And({
                    let keys = {
                        let len = u.arbitrary_len::<SearchKey>()?;
                        let mut tmp = Vec::with_capacity(len);

                        for _ in 0..len {
                            tmp.push(make_search_key_rec(u, depth - 1)?);
                        }

                        tmp
                    };

                    if !keys.is_empty() {
                        NonEmptyVec::try_from(keys).unwrap()
                    } else {
                        NonEmptyVec::from(make_search_key(u)?)
                    }
                }),
                1 => SearchKey::SequenceSet(SequenceSet::arbitrary(u)?),
                2 => SearchKey::All,
                3 => SearchKey::Answered,
                4 => SearchKey::Bcc(AString::arbitrary(u)?),
                5 => SearchKey::Before(NaiveDate::arbitrary(u)?),
                6 => SearchKey::Body(AString::arbitrary(u)?),
                7 => SearchKey::Cc(AString::arbitrary(u)?),
                8 => SearchKey::Deleted,
                9 => SearchKey::Draft,
                10 => SearchKey::Flagged,
                11 => SearchKey::From(AString::arbitrary(u)?),
                12 => SearchKey::Header(AString::arbitrary(u)?, AString::arbitrary(u)?),
                13 => SearchKey::Keyword(Atom::arbitrary(u)?),
                14 => SearchKey::Larger(u32::arbitrary(u)?),
                15 => SearchKey::New,
                16 => SearchKey::Not(Box::new(make_search_key_rec(u, depth - 1)?)),
                17 => SearchKey::Old,
                18 => SearchKey::On(NaiveDate::arbitrary(u)?),
                19 => SearchKey::Or(
                    Box::new(make_search_key_rec(u, depth - 1)?),
                    Box::new(make_search_key_rec(u, depth - 1)?),
                ),
                20 => SearchKey::Recent,
                21 => SearchKey::Seen,
                22 => SearchKey::SentBefore(NaiveDate::arbitrary(u)?),
                23 => SearchKey::SentOn(NaiveDate::arbitrary(u)?),
                24 => SearchKey::SentSince(NaiveDate::arbitrary(u)?),
                25 => SearchKey::Since(NaiveDate::arbitrary(u)?),
                26 => SearchKey::Smaller(u32::arbitrary(u)?),
                27 => SearchKey::Subject(AString::arbitrary(u)?),
                28 => SearchKey::Text(AString::arbitrary(u)?),
                29 => SearchKey::To(AString::arbitrary(u)?),
                30 => SearchKey::Uid(SequenceSet::arbitrary(u)?),
                31 => SearchKey::Unanswered,
                32 => SearchKey::Undeleted,
                33 => SearchKey::Undraft,
                34 => SearchKey::Unflagged,
                35 => SearchKey::Unkeyword(Atom::arbitrary(u)?),
                36 => SearchKey::Unseen,
                _ => unreachable!(),
            })
        }

        make_search_key_rec(u, 7)
    }
}

impl<'a> Arbitrary<'a> for DateTime {
    fn arbitrary(_: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // FIXME(#30): make arbitrary :-)

        let local_datetime = NaiveDateTime::new(
            ChronoNaiveDate::from_ymd_opt(1985, 2, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
        );

        Ok(DateTime(
            FixedOffset::east_opt(3600)
                .unwrap()
                .from_local_datetime(&local_datetime)
                .unwrap(),
        ))
    }
}

impl<'a> Arbitrary<'a> for NaiveDate {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        loop {
            // This was copied from the `chrono`.
            const MIN_YEAR: i32 = i32::MIN >> 13;
            const MAX_YEAR: i32 = i32::MAX >> 13;

            let year: i32 = u.int_in_range(MIN_YEAR..=MAX_YEAR)?;
            let month: u32 = u.int_in_range(1..=12)?;
            let day: u32 = u.int_in_range(1..=31)?;

            if let Some(chrono_naive_date) = ChronoNaiveDate::from_ymd_opt(year, month, day) {
                return Ok(NaiveDate(chrono_naive_date));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use arbitrary::{Arbitrary, Error, Unstructured};
    use rand::prelude::*;

    use crate::{
        command::Command,
        response::{Greeting, Response},
    };

    /// Note: We could encode/decode/etc. here but only want to exercise the arbitrary logic itself.
    macro_rules! impl_test_arbitrary {
        ($object:ty) => {
            let mut rng = rand::thread_rng();
            let mut data = [0u8; 256];

            // Randomize.
            rng.try_fill(&mut data).unwrap();
            let mut unstructured = Unstructured::new(&data);

            let mut count = 0;
            loop {
                match <$object>::arbitrary(&mut unstructured) {
                    Ok(_out) => {
                        count += 1;

                        // println!("{:?}", _out);

                        if count >= 1_000 {
                            break;
                        }
                    }
                    Err(Error::IncorrectFormat) => {
                        // Randomize.
                        rng.try_fill(&mut data).unwrap();
                        unstructured = Unstructured::new(&data);
                    }
                    Err(Error::NotEnoughData | Error::EmptyChoose) => {
                        unreachable!();
                    }
                    Err(_) => {
                        unimplemented!()
                    }
                }
            }
        };
    }

    #[test]
    fn test_arbitrary_greeting() {
        impl_test_arbitrary! {Greeting};
    }

    #[test]
    fn test_arbitrary_command() {
        impl_test_arbitrary! {Command};
    }

    #[test]
    fn test_arbitrary_response() {
        impl_test_arbitrary! {Response};
    }
}
