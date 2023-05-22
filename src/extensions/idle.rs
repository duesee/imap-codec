//! IMAP4 IDLE command
//!
//! This extension enables the [CommandBody::Idle](crate::types::command::CommandBody#variant.Idle) variant.
//! No additional types are used.

// Additional changes:
//
// command_auth =/ idle

use std::io::Write;

use imap_types::command::{idle::IdleDone, CommandBody};
use nom::{bytes::streaming::tag_no_case, combinator::value, IResult};

use crate::codec::Encode;

/// `idle = "IDLE" CRLF "DONE"` (edited)
///
/// ```text
/// idle = "IDLE" CRLF "DONE"
///        ^^^^^^^^^^^
///        |
///        This is parsed here.
///        CRLF is consumed in upper command parser.
/// ```
///
/// Valid only in Authenticated or Selected state
pub fn idle(input: &[u8]) -> IResult<&[u8], CommandBody> {
    value(CommandBody::Idle, tag_no_case("IDLE"))(input)
}

/// `idle = "IDLE" CRLF "DONE"` (edited)
///
/// ```text
/// idle = "IDLE" CRLF "DONE" CRLF
///                    ^^^^^^^^^^^
///                    |
///                    This is parsed here.
///                    CRLF is additionally consumed in this parser.
/// ```
///
/// Valid only in Authenticated or Selected state
///
/// Note: This parser must be executed *instead* of the command parser
/// when the server is in the IDLE state.
pub fn idle_done(input: &[u8]) -> IResult<&[u8], IdleDone> {
    value(IdleDone, tag_no_case("DONE\r\n"))(input)
}

impl Encode for IdleDone {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"DONE\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{command::CommandBody, testing::known_answer_test_encode};

    #[test]
    fn test_encode_command_body_idle() {
        let tests = [(CommandBody::Idle, b"IDLE".as_ref())];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_encode_idle_done() {
        let tests = [(IdleDone, b"DONE\r\n".as_ref())];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
