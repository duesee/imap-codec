use crate::{
    parse::{
        address::address,
        core::{nil, nstring},
    },
    types::{address::Address, core::NString, envelope::Envelope},
};
use abnf_core::streaming::SP;
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
pub(crate) fn envelope(input: &[u8]) -> IResult<&[u8], Envelope> {
    let parser = delimited(
        tag(b"("),
        tuple((
            env_date,
            SP,
            env_subject,
            SP,
            env_from,
            SP,
            env_sender,
            SP,
            env_reply_to,
            SP,
            env_to,
            SP,
            env_cc,
            SP,
            env_bcc,
            SP,
            env_in_reply_to,
            SP,
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
fn env_date(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// env-subject = nstring
fn env_subject(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// env-from = "(" 1*address ")" / nil
fn env_from(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-sender = "(" 1*address ")" / nil
fn env_sender(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-reply-to = "(" 1*address ")" / nil
fn env_reply_to(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-to = "(" 1*address ")" / nil
fn env_to(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-cc = "(" 1*address ")" / nil
fn env_cc(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-bcc = "(" 1*address ")" / nil
fn env_bcc(input: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// env-in-reply-to = nstring
fn env_in_reply_to(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// env-message-id = nstring
fn env_message_id(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}
