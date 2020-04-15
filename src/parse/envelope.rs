use crate::types::response::{Address, Envelope};
use crate::{
    parse::{
        address::address,
        core::{nil, nstring},
        sp,
    },
    types::core::NString,
};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    combinator::map,
    multi::many1,
    sequence::{delimited, tuple},
    IResult,
};

/// envelope = "("
///            env-date SP
///            env-subject SP
///            env-from SP
///            env-sender SP
///            env-reply-to SP
///            env-to SP
///            env-cc SP
///            env-bcc SP
///            env-in-reply-to SP
///            env-message-id
///            ")"
pub fn envelope(input: &[u8]) -> IResult<&[u8], Envelope> {
    let parser = delimited(
        tag(b"("),
        tuple((
            env_date,
            sp,
            env_subject,
            sp,
            env_from,
            sp,
            env_sender,
            sp,
            env_reply_to,
            sp,
            env_to,
            sp,
            env_cc,
            sp,
            env_bcc,
            sp,
            env_in_reply_to,
            sp,
            env_message_id,
        )),
        tag(b")"),
    );

    let (
        remaining,
        (
            date,
            _,
            subject,
            _,
            from,
            _,
            sender,
            _,
            reply_to,
            _,
            to,
            _,
            cc,
            _,
            bcc,
            _,
            in_reply_to,
            _,
            message_id,
        ),
    ) = parser(input)?;

    Ok((
        remaining,
        Envelope {
            date,
            subject,
            from,
            sender,
            reply_to,
            to,
            cc,
            bcc,
            in_reply_to,
            message_id,
        },
    ))
}

/// env-date = nstring
pub fn env_date(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// env-subject = nstring
pub fn env_subject(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// env-from = "(" 1*address ")" / nil
pub fn env_from(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-sender = "(" 1*address ")" / nil
pub fn env_sender(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-reply-to = "(" 1*address ")" / nil
pub fn env_reply_to(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-to = "(" 1*address ")" / nil
pub fn env_to(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-cc = "(" 1*address ")" / nil
pub fn env_cc(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-bcc = "(" 1*address ")" / nil
pub fn env_bcc(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-in-reply-to = nstring
pub fn env_in_reply_to(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// env-message-id = nstring
pub fn env_message_id(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}
