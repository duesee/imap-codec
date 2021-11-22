use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    sequence::{delimited, tuple},
    IResult,
};

use crate::{
    parse::{
        core::{number, nz_number},
        section::section,
    },
    types::fetch_attributes::FetchAttribute,
};

/// fetch-att = "ENVELOPE" /
///             "FLAGS" /
///             "INTERNALDATE" /
///             "RFC822" [".HEADER" / ".SIZE" / ".TEXT"] /
///             "BODY" ["STRUCTURE"] /
///             "UID" /
///             "BODY" section ["<" number "." nz-number ">"] /
///             "BODY.PEEK" section ["<" number "." nz-number ">"]
pub(crate) fn fetch_att(input: &[u8]) -> IResult<&[u8], FetchAttribute> {
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
