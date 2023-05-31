//! The IMAP ENABLE Extension

// Additional changes:
//
// capability    =/ "ENABLE"
// command-any   =/ "ENABLE" 1*(SP capability)
// response-data =/ "*" SP enable-data CRLF

use std::io::Write;

use abnf_core::streaming::SP;
/// Re-export everything from imap-types.
pub use imap_types::extensions::enable::*;
use nom::{
    bytes::streaming::tag_no_case,
    combinator::map,
    multi::{many0, many1},
    sequence::{preceded, tuple},
    IResult,
};

use crate::{
    codec::{CoreEncode, EncodeContext},
    command::CommandBody,
    core::atom,
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

impl<'a> CoreEncode for CapabilityEnable<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Utf8(Utf8Kind::Accept) => writer.write_all(b"UTF8=ACCEPT"),
            Self::Utf8(Utf8Kind::Only) => writer.write_all(b"UTF8=ONLY"),
            #[cfg(feature = "ext_condstore_qresync")]
            Self::CondStore => writer.write_all(b"CONDSTORE"),
            Self::Other(other) => other.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for CapabilityEnableOther<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.inner().core_encode(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{command::Command, core::Atom, testing::kat_inverse_command};

    #[test]
    fn test_parse_enable() {
        let got = enable(b"enable UTF8=ACCEPT\r\n").unwrap().1;
        assert_eq!(
            CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Accept)]).unwrap(),
            got
        );
    }

    #[test]
    fn test_kat_inverse_command_enable() {
        kat_inverse_command(&[
            (
                b"A ENABLE UTF8=ONLY\r\n".as_ref(),
                b"".as_ref(),
                Command::new(
                    "A",
                    CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Only)]).unwrap(),
                )
                .unwrap(),
            ),
            (
                b"A ENABLE UTF8=ACCEPT\r\n?",
                b"?".as_ref(),
                Command::new(
                    "A",
                    CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Accept)]).unwrap(),
                )
                .unwrap(),
            ),
            (
                b"A ENABLE FOO\r\n??",
                b"??",
                Command::new(
                    "A",
                    CommandBody::enable(vec![CapabilityEnable::Other(
                        CapabilityEnableOther::try_from(Atom::try_from("FOO").unwrap()).unwrap(),
                    )])
                    .unwrap(),
                )
                .unwrap(),
            ),
        ]);
    }
}
