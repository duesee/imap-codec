//! The IMAP ENABLE Extension

// Additional changes:
//
// capability    =/ "ENABLE"
// command-any   =/ "ENABLE" 1*(SP capability)
// response-data =/ "*" SP enable-data CRLF

use std::convert::TryInto;

use abnf_core::streaming::SP;
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    multi::{many0, many1},
    sequence::{preceded, tuple},
    IResult,
};

use crate::{
    core::atom,
    types::{
        command::CommandBody,
        extensions::rfc5161::{CapabilityEnable, Utf8Kind},
        response::Data,
    },
};

/// `command-any =/ "ENABLE" 1*(SP capability)`
///
/// Note:
///
/// Introduced into imap-codec as ...
///
/// ```text
/// enable = "ENABLE" 1*(SP capability)
///
/// command-any =/ enable
/// ```
pub fn enable(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case("ENABLE"),
        many1(preceded(SP, capability_enable)),
    ));

    let (remaining, (_, capabilities)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Enable {
            capabilities: capabilities.try_into().unwrap(),
        },
    ))
}

pub fn capability_enable(input: &[u8]) -> IResult<&[u8], CapabilityEnable> {
    alt((
        value(
            CapabilityEnable::Utf8(Utf8Kind::Accept),
            tag_no_case(b"UTF8=ACCEPT"),
        ),
        value(
            CapabilityEnable::Utf8(Utf8Kind::Only),
            tag_no_case(b"UTF8=ONLY"),
        ),
        map(atom, CapabilityEnable::Other),
    ))(input)
}

/// `enable-data = "ENABLED" *(SP capability)`
pub fn enable_data(input: &[u8]) -> IResult<&[u8], Data> {
    let mut parser = tuple((
        tag_no_case(b"ENABLED"),
        many0(preceded(SP, capability_enable)),
    ));

    let (remaining, (_, capabilities)) = parser(input)?;

    Ok((remaining, { Data::Enabled { capabilities } }))
}
