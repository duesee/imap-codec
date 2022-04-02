//! IMAP4 IDLE command
//!
//! This extension enables the [CommandBody::Idle](crate::types::command::CommandBody#variant.Idle) variant.
//! No additional types are used.

// Additional changes:
//
// command_auth =/ idle

// pub mod types {
//
// }

pub(crate) mod parse {
    use abnf_core::streaming::CRLF;
    use nom::{bytes::streaming::tag_no_case, combinator::value, sequence::tuple, IResult};

    use crate::types::command::CommandBody;

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
    ///
    // TODO: just interpret as command?
    pub fn idle_done(input: &[u8]) -> IResult<&[u8], ()> {
        let mut parser = value((), tuple((tag_no_case("DONE"), CRLF)));

        let (remaining, parsed_idle_done) = parser(input)?;

        Ok((remaining, parsed_idle_done))
    }
}
