//! The IMAP ENABLE Extension

// Additional changes:
//
// capability    =/ "ENABLE"
// command-any   =/ "ENABLE" 1*(SP capability)
// response-data =/ "*" SP enable-data CRLF

use std::io::Write;

use abnf_core::streaming::SP;
use imap_types::{extensions::enable::CapabilityEnableOther, message::Utf8Kind};
use nom::{
    bytes::streaming::tag_no_case,
    combinator::map,
    multi::{many0, many1},
    sequence::{preceded, tuple},
    IResult,
};

use crate::{
    codec::Encode, command::CommandBody, imap4rev1::core::atom, message::CapabilityEnable,
    response::Data,
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
    map(atom, CapabilityEnable::from)(input)
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

impl<'a> Encode for CapabilityEnable<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Utf8(Utf8Kind::Accept) => writer.write_all(b"UTF8=ACCEPT"),
            Self::Utf8(Utf8Kind::Only) => writer.write_all(b"UTF8=ONLY"),
            Self::Other(other) => other.encode(writer),
        }
    }
}

impl<'a> Encode for CapabilityEnableOther<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.inner().encode(writer)
    }
}

#[cfg(test)]
mod tests {
    use imap_types::core::{Atom, NonEmptyVec, NonEmptyVecError};

    use super::*;

    #[test]
    fn test_encode_command_body_enable() {
        let tests = [
            (
                CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Only)]),
                Ok((
                    CommandBody::Enable {
                        capabilities: NonEmptyVec::from(CapabilityEnable::Utf8(Utf8Kind::Only)),
                    },
                    b"ENABLE UTF8=ONLY".as_ref(),
                )),
            ),
            (
                CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Accept)]),
                Ok((
                    CommandBody::Enable {
                        capabilities: NonEmptyVec::from(CapabilityEnable::Utf8(Utf8Kind::Accept)),
                    },
                    b"ENABLE UTF8=ACCEPT",
                )),
            ),
            (
                CommandBody::enable(vec![CapabilityEnable::Other(
                    CapabilityEnableOther::try_from(Atom::try_from("FOO").unwrap()).unwrap(),
                )]),
                Ok((
                    CommandBody::Enable {
                        capabilities: NonEmptyVec::from(CapabilityEnable::Other(
                            CapabilityEnableOther::try_from(Atom::try_from("FOO").unwrap())
                                .unwrap(),
                        )),
                    },
                    b"ENABLE FOO",
                )),
            ),
            (CommandBody::enable(vec![]), Err(NonEmptyVecError::Empty)),
        ];

        for (test, expected) in tests {
            match test {
                Ok(got) => {
                    let bytes = got.encode_detached().unwrap();
                    assert_eq!(expected, Ok((got, bytes.as_ref())));
                }
                Err(got) => {
                    assert_eq!(Err(got), expected);
                }
            }
        }
    }

    #[test]
    fn test_parse_enable() {
        use imap_types::message::{CapabilityEnable, Utf8Kind};

        let got = enable(b"enable UTF8=ACCEPT\r\n").unwrap().1;
        assert_eq!(
            CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Accept)]).unwrap(),
            got
        );
    }
}
