use std::num::NonZeroU32;

use abnf_core::streaming::SP;
use imap_types::{
    command::fetch::FetchAttribute, core::NonEmptyVec, response::data::FetchAttributeValue,
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

use crate::rfc3501::{
    body::body,
    core::{nstring, number, nz_number},
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
pub fn fetch_att(input: &[u8]) -> IResult<&[u8], FetchAttribute> {
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
pub fn msg_att(input: &[u8]) -> IResult<&[u8], NonEmptyVec<FetchAttributeValue>> {
    delimited(
        tag(b"("),
        map(
            separated_list1(SP, alt((msg_att_dynamic, msg_att_static))),
            |attrs| NonEmptyVec::new_unchecked(attrs),
        ),
        tag(b")"),
    )(input)
}

/// `msg-att-dynamic = "FLAGS" SP "(" [flag-fetch *(SP flag-fetch)] ")"`
///
/// Note: MAY change for a message
pub fn msg_att_dynamic(input: &[u8]) -> IResult<&[u8], FetchAttributeValue> {
    let mut parser = tuple((
        tag_no_case(b"FLAGS"),
        SP,
        delimited(tag(b"("), opt(separated_list1(SP, flag_fetch)), tag(b")")),
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
pub fn msg_att_static(input: &[u8]) -> IResult<&[u8], FetchAttributeValue> {
    alt((
        map(
            tuple((tag_no_case(b"ENVELOPE"), SP, envelope)),
            |(_, _, envelope)| FetchAttributeValue::Envelope(envelope),
        ),
        map(
            tuple((tag_no_case(b"INTERNALDATE"), SP, date_time)),
            |(_, _, date_time)| FetchAttributeValue::InternalDate(date_time),
        ),
        map(
            tuple((tag_no_case(b"RFC822.HEADER"), SP, nstring)),
            |(_, _, nstring)| FetchAttributeValue::Rfc822Header(nstring),
        ),
        map(
            tuple((tag_no_case(b"RFC822.TEXT"), SP, nstring)),
            |(_, _, nstring)| FetchAttributeValue::Rfc822Text(nstring),
        ),
        map(
            tuple((tag_no_case(b"RFC822.SIZE"), SP, number)),
            |(_, _, num)| FetchAttributeValue::Rfc822Size(num),
        ),
        map(
            tuple((tag_no_case(b"RFC822"), SP, nstring)),
            |(_, _, nstring)| FetchAttributeValue::Rfc822(nstring),
        ),
        map(
            tuple((tag_no_case(b"BODYSTRUCTURE"), SP, body(8))),
            |(_, _, body)| FetchAttributeValue::BodyStructure(body),
        ),
        map(
            tuple((tag_no_case(b"BODY"), SP, body(8))),
            |(_, _, body)| FetchAttributeValue::Body(body),
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(delimited(tag(b"<"), number, tag(b">"))),
                SP,
                nstring,
            )),
            |(_, section, origin, _, data)| FetchAttributeValue::BodyExt {
                section,
                origin,
                data,
            },
        ),
        map(tuple((tag_no_case(b"UID"), SP, uniqueid)), |(_, _, uid)| {
            FetchAttributeValue::Uid(uid)
        }),
    ))(input)
}

#[inline]
/// `uniqueid = nz-number`
///
/// Note: Strictly ascending
pub fn uniqueid(input: &[u8]) -> IResult<&[u8], NonZeroU32> {
    nz_number(input)
}
