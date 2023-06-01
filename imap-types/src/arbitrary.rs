use arbitrary::{Arbitrary, Unstructured};
use chrono::{FixedOffset, TimeZone};

#[cfg(feature = "ext_enable")]
use crate::extensions::enable::CapabilityEnableOther;
#[cfg(feature = "ext_quota")]
use crate::extensions::quota::ResourceOther;
use crate::{
    auth::AuthMechanismOther,
    body::{
        BasicFields, Body, BodyExtension, BodyStructure, MultiPartExtensionData,
        SinglePartExtensionData, SpecificFields,
    },
    core::{
        AString, Atom, AtomExt, IString, Literal, NString, NonEmptyVec, Quoted, QuotedChar, Tag,
        Text,
    },
    datetime::{DateTime, NaiveDate},
    envelope::Envelope,
    flag::FlagExtension,
    mailbox::{ListCharString, Mailbox, MailboxOther},
    response::{Capability, Code, CodeOther, ContinueBasic},
    search::SearchKey,
    sequence::SequenceSet,
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

impl<'a> Arbitrary<'a> for ContinueBasic<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        ContinueBasic::new(Option::<Code>::arbitrary(u)?, Text::arbitrary(u)?)
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

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
        Ok(CodeOther::unchecked(b"IMAP-CODEC-CODE-OTHER>".as_ref()))
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

impl<'a> Arbitrary<'a> for BodyStructure<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        fn make_body_structure_terminator<'a>(
            u: &mut Unstructured<'a>,
        ) -> arbitrary::Result<BodyStructure<'a>> {
            Ok(BodyStructure::Single {
                body: Body {
                    basic: BasicFields::arbitrary(u)?,
                    specific: match u.int_in_range(1..=2)? {
                        1 => SpecificFields::Basic {
                            r#type: IString::arbitrary(u)?,
                            subtype: IString::arbitrary(u)?,
                        },
                        // No SpecificFields::Message because it would recurse.
                        2 => SpecificFields::Text {
                            subtype: IString::arbitrary(u)?,
                            number_of_lines: u32::arbitrary(u)?,
                        },
                        _ => unreachable!(),
                    },
                },
                extension_data: Option::<SinglePartExtensionData>::arbitrary(u)?,
            })
        }

        fn make_body_structure_rec<'a>(
            u: &mut Unstructured<'a>,
            depth: u8,
        ) -> arbitrary::Result<BodyStructure<'a>> {
            if depth == 0 {
                return make_body_structure_terminator(u);
            }

            Ok(match u.int_in_range(1..=2)? {
                1 => BodyStructure::Single {
                    body: Body {
                        basic: BasicFields::arbitrary(u)?,
                        specific: match u.int_in_range(1..=3)? {
                            1 => SpecificFields::Basic {
                                r#type: IString::arbitrary(u)?,
                                subtype: IString::arbitrary(u)?,
                            },
                            2 => SpecificFields::Message {
                                envelope: Box::<Envelope>::arbitrary(u)?,
                                body_structure: Box::new(make_body_structure_rec(u, depth - 1)?),
                                number_of_lines: u32::arbitrary(u)?,
                            },
                            3 => SpecificFields::Text {
                                subtype: IString::arbitrary(u)?,
                                number_of_lines: u32::arbitrary(u)?,
                            },
                            _ => unreachable!(),
                        },
                    },
                    extension_data: Option::<SinglePartExtensionData>::arbitrary(u)?,
                },
                2 => BodyStructure::Multi {
                    bodies: {
                        let bodies = {
                            let len = u.arbitrary_len::<BodyStructure>()?;
                            let mut tmp = Vec::with_capacity(len);

                            for _ in 0..len {
                                tmp.push(make_body_structure_rec(u, depth - 1)?);
                            }

                            tmp
                        };

                        if !bodies.is_empty() {
                            NonEmptyVec::try_from(bodies).unwrap()
                        } else {
                            NonEmptyVec::from(make_body_structure_terminator(u)?)
                        }
                    },
                    subtype: IString::arbitrary(u)?,
                    extension_data: Option::<MultiPartExtensionData>::arbitrary(u)?,
                },
                _ => unreachable!(),
            })
        }

        make_body_structure_rec(u, 3)
    }
}

impl<'a> Arbitrary<'a> for BodyExtension<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        fn make_body_extension_terminator<'a>(
            u: &mut Unstructured<'a>,
        ) -> arbitrary::Result<BodyExtension<'a>> {
            Ok(match u.int_in_range(1..=2)? {
                1 => BodyExtension::NString(NString::arbitrary(u)?),
                2 => BodyExtension::Number(u32::arbitrary(u)?),
                // No `BodyExtension::List` because it could recurse.
                _ => unreachable!(),
            })
        }

        fn make_body_extension_rec<'a>(
            u: &mut Unstructured<'a>,
            depth: u8,
        ) -> arbitrary::Result<BodyExtension<'a>> {
            if depth == 0 {
                return make_body_extension_terminator(u);
            }

            Ok(match u.int_in_range(1..=2)? {
                1 => BodyExtension::NString(NString::arbitrary(u)?),
                2 => BodyExtension::Number(u32::arbitrary(u)?),
                3 => BodyExtension::List({
                    let body_extensions = {
                        let len = u.arbitrary_len::<BodyExtension>()?;
                        let mut tmp = Vec::with_capacity(len);

                        for _ in 0..len {
                            tmp.push(make_body_extension_rec(u, depth - 1)?);
                        }

                        tmp
                    };

                    if !body_extensions.is_empty() {
                        NonEmptyVec::try_from(body_extensions).unwrap()
                    } else {
                        NonEmptyVec::from(make_body_extension_terminator(u)?)
                    }
                }),
                _ => unreachable!(),
            })
        }

        make_body_extension_rec(u, 3)
    }
}

impl<'a> Arbitrary<'a> for DateTime {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Note: `chrono`s `NaiveDate::arbitrary` may `panic!`.
        //       Thus, we implement this manually here.
        let local_datetime = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                u.int_in_range(0..=9999)?,
                u.int_in_range(1..=12)?,
                u.int_in_range(1..=31)?,
            )
            .ok_or(arbitrary::Error::IncorrectFormat)?,
            chrono::NaiveTime::arbitrary(u)?,
        );

        let hours = u.int_in_range(0..=23 * 3600)?;
        let minutes = u.int_in_range(0..=59)? * 60;
        // Seconds must be zero due to IMAPs encoding.

        DateTime::try_from(
            FixedOffset::east_opt(hours + minutes)
                .unwrap()
                .from_local_datetime(&local_datetime)
                .unwrap(),
        )
        .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

impl<'a> Arbitrary<'a> for NaiveDate {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        NaiveDate::try_from(chrono::NaiveDate::arbitrary(u)?)
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod tests {
    use arbitrary::{Arbitrary, Error, Unstructured};
    #[cfg(feature = "bounded-static")]
    use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
    use rand::{rngs::SmallRng, Rng, SeedableRng};

    use crate::{
        command::Command,
        response::{Greeting, Response},
    };

    /// Note: We could encode/decode/etc. here but only want to exercise the arbitrary logic itself.
    macro_rules! impl_test_arbitrary {
        ($object:ty) => {
            let mut rng = SmallRng::seed_from_u64(1337);
            let mut data = [0u8; 256];

            // Randomize.
            rng.try_fill(&mut data).unwrap();
            let mut unstructured = Unstructured::new(&data);

            let mut count = 0;
            loop {
                match <$object>::arbitrary(&mut unstructured) {
                    Ok(_out) => {
                        count += 1;

                        #[cfg(feature = "bounded-static")]
                        {
                            let out_to_static = _out.to_static();
                            assert_eq!(_out, out_to_static);

                            let out_into_static = _out.into_static();
                            assert_eq!(out_to_static, out_into_static);
                        }

                        if count >= 1_000 {
                            break;
                        }
                    }
                    Err(Error::NotEnoughData | Error::IncorrectFormat) => {
                        // Randomize.
                        rng.try_fill(&mut data).unwrap();
                        unstructured = Unstructured::new(&data);
                    }
                    Err(Error::EmptyChoose) => {
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
