use crate::{
    parse::{
        body::body,
        core::{nstring, number, nz_number},
        datetime::date_time,
        envelope::envelope,
        flag::flag_fetch,
        section::section,
    },
    types::response::{Data, DataItemResponse},
};
use abnf_core::streaming::SP;
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt},
    multi::separated_nonempty_list,
    sequence::{delimited, tuple},
    IResult,
};

/// message-data = nz-number SP ("EXPUNGE" / ("FETCH" SP msg-att))
pub fn message_data(input: &[u8]) -> IResult<&[u8], Data> {
    let (remaining, (msg, _)) = tuple((nz_number, SP))(input)?;

    alt((
        map(tag_no_case(b"EXPUNGE"), move |_| Data::Expunge(msg)),
        map(
            tuple((tag_no_case(b"FETCH"), SP, msg_att)),
            move |(_, _, items)| Data::Fetch { msg, items },
        ),
    ))(remaining)
}

/// msg-att = "("
///           (msg-att-dynamic / msg-att-static) *(SP (msg-att-dynamic / msg-att-static))
///           ")"
pub fn msg_att(input: &[u8]) -> IResult<&[u8], Vec<DataItemResponse>> {
    delimited(
        tag(b"("),
        separated_nonempty_list(SP, alt((msg_att_dynamic, msg_att_static))),
        tag(b")"),
    )(input)
}

/// msg-att-dynamic = "FLAGS" SP "(" [flag-fetch *(SP flag-fetch)] ")"
///                     ; MAY change for a message
///
pub fn msg_att_dynamic(input: &[u8]) -> IResult<&[u8], DataItemResponse> {
    let parser = tuple((
        tag_no_case(b"FLAGS"),
        SP,
        delimited(
            tag(b"("),
            opt(separated_nonempty_list(SP, flag_fetch)),
            tag(b")"),
        ),
    ));

    let (remaining, (_, _, flags)) = parser(input)?;

    Ok((
        remaining,
        DataItemResponse::Flags(flags.unwrap_or_default()),
    ))
}

/// msg-att-static = "ENVELOPE" SP envelope /
///                  "INTERNALDATE" SP date-time /
///                  "RFC822" [".HEADER" / ".TEXT"] SP nstring /
///                  "RFC822.SIZE" SP number /
///                  "BODY" ["STRUCTURE"] SP body /
///                  "BODY" section ["<" number ">"] SP nstring /
///                  "UID" SP uniqueid
///                    ; MUST NOT change for a message
pub fn msg_att_static(input: &[u8]) -> IResult<&[u8], DataItemResponse> {
    alt((
        map(
            tuple((tag_no_case(b"ENVELOPE"), SP, envelope)),
            |(_, _, envelope)| DataItemResponse::Envelope(envelope),
        ),
        map(
            // FIXME: do not use unwrap()
            tuple((tag_no_case(b"INTERNALDATE"), SP, date_time)),
            |(_, _, date_time)| DataItemResponse::InternalDate(date_time.unwrap()),
        ),
        alt((
            map(
                tuple((tag_no_case(b"RFC822.HEADER"), SP, nstring)),
                |(_, _, nstring)| DataItemResponse::Rfc822Header(nstring),
            ),
            map(
                tuple((tag_no_case(b"RFC822.TEXT"), SP, nstring)),
                |(_, _, nstring)| DataItemResponse::Rfc822Text(nstring),
            ),
            map(
                tuple((tag_no_case(b"RFC822"), SP, nstring)),
                |(_, _, nstring)| DataItemResponse::Rfc822(nstring),
            ),
        )),
        map(
            tuple((tag_no_case(b"RFC822.SIZE"), SP, number)),
            |(_, _, num)| DataItemResponse::Rfc822Size(num),
        ),
        alt((
            map(
                tuple((tag_no_case(b"BODYSTRUCTURE"), SP, body)),
                |(_, _, _body)| unimplemented!(),
            ),
            map(
                tuple((tag_no_case(b"BODY"), SP, body)),
                |(_, _, _body)| unimplemented!(),
            ),
        )),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(delimited(tag(b"<"), number, tag(b">"))),
                SP,
                nstring,
            )),
            |(_, section, origin, _, data)| DataItemResponse::BodyExt {
                section,
                origin,
                data,
            },
        ),
        map(tuple((tag_no_case(b"UID"), SP, uniqueid)), |(_, _, uid)| {
            DataItemResponse::Uid(uid)
        }),
    ))(input)
}

/// uniqueid = nz-number ; Strictly ascending
pub fn uniqueid(input: &[u8]) -> IResult<&[u8], u32> {
    nz_number(input)
}
