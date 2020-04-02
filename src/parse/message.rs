use crate::parse::{
    body::body,
    core::{nstring, number, nz_number},
    datetime::date_time,
    envelope::envelope,
    flag::flag_fetch,
    section::section,
    sp,
};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    combinator::{map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

/// message-data = nz-number SP ("EXPUNGE" / ("FETCH" SP msg-att))
pub fn message_data(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        nz_number,
        sp,
        alt((
            map(tag_no_case(b"EXPUNGE"), |_| unimplemented!()),
            map(
                tuple((tag_no_case(b"FETCH"), sp, msg_att)),
                |_| unimplemented!(),
            ),
        )),
    ));

    let (_remaining, _parsed_message_data) = parser(input)?;

    unimplemented!();
}

/// msg-att = "(" (msg-att-dynamic / msg-att-static) *(SP (msg-att-dynamic / msg-att-static)) ")"
pub fn msg_att(input: &[u8]) -> IResult<&[u8], ()> {
    // TODO: use separated_list_not_empty
    let parser = tuple((
        tag_no_case(b"("),
        alt((
            map(msg_att_dynamic, |_| unimplemented!()),
            map(msg_att_static, |_| unimplemented!()),
        )),
        many0(tuple((
            sp,
            alt((
                map(msg_att_dynamic, |_| unimplemented!()),
                map(msg_att_static, |_| unimplemented!()),
            )),
        ))),
        tag_no_case(b")"),
    ));

    let (_remaining, _parsed_msg_att) = parser(input)?;

    unimplemented!();
}

/// msg-att-dynamic = "FLAGS" SP "(" [flag-fetch *(SP flag-fetch)] ")"
///                     ; MAY change for a message
///
pub fn msg_att_dynamic(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        tag_no_case(b"FLAGS"),
        sp,
        tag_no_case(b"("),
        opt(tuple((flag_fetch, many0(tuple((sp, flag_fetch)))))),
        tag_no_case(b")"),
    ));

    let (_remaining, _parsed_msg_att_dynamic) = parser(input)?;

    unimplemented!();
}

/// msg-att-static = "ENVELOPE" SP envelope / "INTERNALDATE" SP date-time /
///                  "RFC822" [".HEADER" / ".TEXT"] SP nstring /
///                  "RFC822.SIZE" SP number /
///                  "BODY" ["STRUCTURE"] SP body /
///                  "BODY" section ["<" number ">"] SP nstring /
///                  "UID" SP uniqueid
///                    ; MUST NOT change for a message
pub fn msg_att_static(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            tuple((tag_no_case(b"ENVELOPE"), sp, envelope)),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"INTERNALDATE"), sp, date_time)),
            |_| unimplemented!(),
        ),
        map(
            tuple((
                tag_no_case(b"RFC822"),
                opt(alt((
                    map(tag_no_case(b".HEADER"), |_| unimplemented!()),
                    map(tag_no_case(b".TEXT"), |_| unimplemented!()),
                ))),
                sp,
                nstring,
            )),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"RFC822.SIZE"), sp, number)),
            |_| unimplemented!(),
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                opt(tag_no_case(b"STRUCTURE")),
                sp,
                body,
            )),
            |_| unimplemented!(),
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(tuple((tag_no_case(b"<"), number, tag_no_case(b">")))),
                sp,
                nstring,
            )),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"UID"), sp, uniqueid)),
            |_| unimplemented!(),
        ),
    ));

    let (_remaining, _parsed_msg_att_static) = parser(input)?;

    unimplemented!();
}

/// uniqueid = nz-number ; Strictly ascending
pub fn uniqueid(input: &[u8]) -> IResult<&[u8], u32> {
    nz_number(input)
}
