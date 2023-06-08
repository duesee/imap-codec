use std::num::NonZeroU32;

use abnf_core::streaming::sp;
/// Re-export everything from imap-types.
pub use imap_types::fetch::*;
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, tuple},
};

use crate::{
    body::body,
    codec::IMAPResult,
    core::{nstring, number, nz_number, NonEmptyVec},
    datetime::date_time,
    envelope::envelope,
    flag::flag_fetch,
    section::section,
};

/// `fetch-att = "ENVELOPE" /
///              "FLAGS" /
///              "INTERNALDATE" /
///              "RFC822" [".HEADER" / ".SIZE" / ".TEXT"] /
///              "BODY" ["STRUCTURE"] /
///              "UID" /
///              "BODY" section ["<" number "." nz-number ">"] /
///              "BODY.PEEK" section ["<" number "." nz-number ">"]`
pub fn fetch_att(input: &[u8]) -> IMAPResult<&[u8], FetchAttribute> {
    alt((
        value(FetchAttribute::Envelope, tag_no_case(b"ENVELOPE")),
        value(FetchAttribute::Flags, tag_no_case(b"FLAGS")),
        value(FetchAttribute::InternalDate, tag_no_case(b"INTERNALDATE")),
        value(FetchAttribute::BodyStructure, tag_no_case(b"BODYSTRUCTURE")),
        map(
            tuple((
                tag_no_case(b"BODY.PEEK"),
                section,
                opt(delimited(
                    tag(b"<"),
                    tuple((number, tag(b"."), nz_number)),
                    tag(b">"),
                )),
            )),
            |(_, section, byterange)| FetchAttribute::BodyExt {
                section,
                partial: byterange.map(|(start, _, end)| (start, end)),
                peek: true,
            },
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(delimited(
                    tag(b"<"),
                    tuple((number, tag(b"."), nz_number)),
                    tag(b">"),
                )),
            )),
            |(_, section, byterange)| FetchAttribute::BodyExt {
                section,
                partial: byterange.map(|(start, _, end)| (start, end)),
                peek: false,
            },
        ),
        value(FetchAttribute::Body, tag_no_case(b"BODY")),
        value(FetchAttribute::Uid, tag_no_case(b"UID")),
        value(FetchAttribute::Rfc822Header, tag_no_case(b"RFC822.HEADER")),
        value(FetchAttribute::Rfc822Size, tag_no_case(b"RFC822.SIZE")),
        value(FetchAttribute::Rfc822Text, tag_no_case(b"RFC822.TEXT")),
        value(FetchAttribute::Rfc822, tag_no_case(b"RFC822")),
    ))(input)
}

/// `msg-att = "("
///            (msg-att-dynamic / msg-att-static) *(SP (msg-att-dynamic / msg-att-static))
///            ")"`
pub fn msg_att(input: &[u8]) -> IMAPResult<&[u8], NonEmptyVec<FetchAttributeValue>> {
    delimited(
        tag(b"("),
        map(
            separated_list1(sp, alt((msg_att_dynamic, msg_att_static))),
            NonEmptyVec::unchecked,
        ),
        tag(b")"),
    )(input)
}

/// `msg-att-dynamic = "FLAGS" SP "(" [flag-fetch *(SP flag-fetch)] ")"`
///
/// Note: MAY change for a message
pub fn msg_att_dynamic(input: &[u8]) -> IMAPResult<&[u8], FetchAttributeValue> {
    let mut parser = tuple((
        tag_no_case(b"FLAGS"),
        sp,
        delimited(tag(b"("), opt(separated_list1(sp, flag_fetch)), tag(b")")),
    ));

    let (remaining, (_, _, flags)) = parser(input)?;

    Ok((
        remaining,
        FetchAttributeValue::Flags(flags.unwrap_or_default()),
    ))
}

/// `msg-att-static = "ENVELOPE" SP envelope /
///                   "INTERNALDATE" SP date-time /
///                   "RFC822" [".HEADER" / ".TEXT"] SP nstring /
///                   "RFC822.SIZE" SP number /
///                   "BODY" ["STRUCTURE"] SP body /
///                   "BODY" section ["<" number ">"] SP nstring /
///                   "UID" SP uniqueid`
///
/// Note: MUST NOT change for a message
pub fn msg_att_static(input: &[u8]) -> IMAPResult<&[u8], FetchAttributeValue> {
    alt((
        map(
            tuple((tag_no_case(b"ENVELOPE"), sp, envelope)),
            |(_, _, envelope)| FetchAttributeValue::Envelope(envelope),
        ),
        map(
            tuple((tag_no_case(b"INTERNALDATE"), sp, date_time)),
            |(_, _, date_time)| FetchAttributeValue::InternalDate(date_time),
        ),
        map(
            tuple((tag_no_case(b"RFC822.HEADER"), sp, nstring)),
            |(_, _, nstring)| FetchAttributeValue::Rfc822Header(nstring),
        ),
        map(
            tuple((tag_no_case(b"RFC822.TEXT"), sp, nstring)),
            |(_, _, nstring)| FetchAttributeValue::Rfc822Text(nstring),
        ),
        map(
            tuple((tag_no_case(b"RFC822.SIZE"), sp, number)),
            |(_, _, num)| FetchAttributeValue::Rfc822Size(num),
        ),
        map(
            tuple((tag_no_case(b"RFC822"), sp, nstring)),
            |(_, _, nstring)| FetchAttributeValue::Rfc822(nstring),
        ),
        map(
            tuple((tag_no_case(b"BODYSTRUCTURE"), sp, body(8))),
            |(_, _, body)| FetchAttributeValue::BodyStructure(body),
        ),
        map(
            tuple((tag_no_case(b"BODY"), sp, body(8))),
            |(_, _, body)| FetchAttributeValue::Body(body),
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(delimited(tag(b"<"), number, tag(b">"))),
                sp,
                nstring,
            )),
            |(_, section, origin, _, data)| FetchAttributeValue::BodyExt {
                section,
                origin,
                data,
            },
        ),
        map(tuple((tag_no_case(b"UID"), sp, uniqueid)), |(_, _, uid)| {
            FetchAttributeValue::Uid(uid)
        }),
    ))(input)
}

#[inline]
/// `uniqueid = nz-number`
///
/// Note: Strictly ascending
pub fn uniqueid(input: &[u8]) -> IMAPResult<&[u8], NonZeroU32> {
    nz_number(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        body::{BasicFields, Body, BodyStructure, SpecificFields},
        core::{IString, NString},
        datetime::DateTime,
        envelope::Envelope,
        testing::known_answer_test_encode,
    };

    #[test]
    fn test_encode_fetch_attribute() {
        let tests = [
            (FetchAttribute::Body, b"BODY".as_ref()),
            (
                FetchAttribute::BodyExt {
                    section: None,
                    partial: None,
                    peek: false,
                },
                b"BODY[]",
            ),
            (FetchAttribute::BodyStructure, b"BODYSTRUCTURE"),
            (FetchAttribute::Envelope, b"ENVELOPE"),
            (FetchAttribute::Flags, b"FLAGS"),
            (FetchAttribute::InternalDate, b"INTERNALDATE"),
            (FetchAttribute::Rfc822, b"RFC822"),
            (FetchAttribute::Rfc822Header, b"RFC822.HEADER"),
            (FetchAttribute::Rfc822Size, b"RFC822.SIZE"),
            (FetchAttribute::Rfc822Text, b"RFC822.TEXT"),
            (FetchAttribute::Uid, b"UID"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_encode_fetch_attribute_value() {
        let tests = [
            (
                FetchAttributeValue::Body(BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString(None),
                            content_transfer_encoding: IString::try_from("base64").unwrap(),
                            size: 42,
                        },
                        specific: SpecificFields::Text {
                            subtype: IString::try_from("foo").unwrap(),
                            number_of_lines: 1337,
                        },
                    },
                    extension_data: None,
                }),
                b"BODY (\"TEXT\" \"foo\" NIL NIL NIL \"base64\" 42 1337)".as_ref(),
            ),
            (
                FetchAttributeValue::BodyExt {
                    section: None,
                    origin: None,
                    data: NString(None),
                },
                b"BODY[] NIL",
            ),
            (
                FetchAttributeValue::BodyExt {
                    section: None,
                    origin: Some(123),
                    data: NString(None),
                },
                b"BODY[]<123> NIL",
            ),
            (
                FetchAttributeValue::BodyStructure(BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString(None),
                            content_transfer_encoding: IString::try_from("base64").unwrap(),
                            size: 213,
                        },
                        specific: SpecificFields::Text {
                            subtype: IString::try_from("").unwrap(),
                            number_of_lines: 224,
                        },
                    },
                    extension_data: None,
                }),
                b"BODYSTRUCTURE (\"TEXT\" \"\" NIL NIL NIL \"base64\" 213 224)",
            ),
            (
                FetchAttributeValue::Envelope(Envelope {
                    date: NString(None),
                    subject: NString(None),
                    from: vec![],
                    sender: vec![],
                    reply_to: vec![],
                    to: vec![],
                    cc: vec![],
                    bcc: vec![],
                    in_reply_to: NString(None),
                    message_id: NString(None),
                }),
                b"ENVELOPE (NIL NIL NIL NIL NIL NIL NIL NIL NIL NIL)",
            ),
            (FetchAttributeValue::Flags(vec![]), b"FLAGS ()"),
            (
                FetchAttributeValue::InternalDate(
                    DateTime::try_from(
                        chrono::DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200")
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                b"INTERNALDATE \"01-Jul-2003 10:52:37 +0200\"",
            ),
            (FetchAttributeValue::Rfc822(NString(None)), b"RFC822 NIL"),
            (
                FetchAttributeValue::Rfc822Header(NString(None)),
                b"RFC822.HEADER NIL",
            ),
            (FetchAttributeValue::Rfc822Size(3456), b"RFC822.SIZE 3456"),
            (
                FetchAttributeValue::Rfc822Text(NString(None)),
                b"RFC822.TEXT NIL",
            ),
            (
                FetchAttributeValue::Uid(NonZeroU32::try_from(u32::MAX).unwrap()),
                b"UID 4294967295",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
