//! # Decoding of messages.
//!
//! You can use [`Decoder`]s to parse messages.
//!
//! IMAP literals make separating the parsing logic from the application logic difficult.
//! When a server recognizes a literal (e.g. `{42}\r\n`) in a command, it first needs to agree to receive more data by sending a so-called "command continuation request" (`+ ...`).
//! Without a command continuation request, a client won't send more data, and the command parser on the server would always return `LiteralFound { length: 42, .. }`.
//! This makes real-world decoding of IMAP more elaborate.
//!
//! Have a look at the [parse_command](https://github.com/duesee/imap-codec/blob/main/imap-codec/examples/parse_command.rs) example to see how a real-world application could decode IMAP.

use std::num::{ParseIntError, TryFromIntError};

#[cfg(feature = "bounded-static")]
use bounded_static::{IntoBoundedStatic, ToStatic};
use imap_types::{
    auth::AuthenticateData,
    command::Command,
    core::{LiteralMode, Tag},
    extensions::idle::IdleDone,
    response::{Greeting, Response},
};
use nom::error::{ErrorKind, FromExternalError, ParseError};

use crate::{
    auth::authenticate_data,
    command::command,
    extensions::idle::idle_done,
    response::{greeting, response},
    AuthenticateDataCodec, CommandCodec, GreetingCodec, IdleDoneCodec, ResponseCodec,
};

/// An extended version of [`nom::IResult`].
pub(crate) type IMAPResult<'a, I, O> = Result<(I, O), nom::Err<IMAPParseError<'a, I>>>;

/// An extended version of [`nom::error::Error`].
#[derive(Debug)]
pub(crate) struct IMAPParseError<'a, I> {
    #[allow(unused)]
    pub input: I,
    pub kind: IMAPErrorKind<'a>,
}

/// An extended version of [`nom::error::ErrorKind`].
#[derive(Debug)]
pub(crate) enum IMAPErrorKind<'a> {
    Literal {
        tag: Option<Tag<'a>>,
        length: u32,
        mode: LiteralMode,
    },
    BadNumber,
    BadBase64,
    BadDateTime,
    LiteralContainsNull,
    RecursionLimitExceeded,
    Nom(ErrorKind),
}

impl<'a, I> ParseError<I> for IMAPParseError<'a, I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::Nom(kind),
        }
    }

    fn append(input: I, kind: ErrorKind, _: Self) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::Nom(kind),
        }
    }
}

impl<'a, I> FromExternalError<I, ParseIntError> for IMAPParseError<'a, I> {
    fn from_external_error(input: I, _: ErrorKind, _: ParseIntError) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::BadNumber,
        }
    }
}

impl<'a, I> FromExternalError<I, TryFromIntError> for IMAPParseError<'a, I> {
    fn from_external_error(input: I, _: ErrorKind, _: TryFromIntError) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::BadNumber,
        }
    }
}

impl<'a, I> FromExternalError<I, base64::DecodeError> for IMAPParseError<'a, I> {
    fn from_external_error(input: I, _: ErrorKind, _: base64::DecodeError) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::BadBase64,
        }
    }
}

/// Decoder.
///
/// Implemented for types that know how to decode a specific IMAP message. See [implementors](trait.Decoder.html#implementors).
pub trait Decoder {
    type Message<'a>: Sized;
    type Error<'a>;

    fn decode<'a>(&self, input: &'a [u8])
        -> Result<(&'a [u8], Self::Message<'a>), Self::Error<'a>>;

    #[cfg(feature = "bounded-static")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bounded-static")))]
    fn decode_static<'a>(
        &self,
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self::Message<'static>), Self::Error<'static>>
    where
        Self::Message<'a>: IntoBoundedStatic<Static = Self::Message<'static>>,
        Self::Error<'a>: IntoBoundedStatic<Static = Self::Error<'static>>,
    {
        let (remaining, value) = self.decode(input).map_err(IntoBoundedStatic::into_static)?;
        Ok((remaining, value.into_static()))
    }
}

/// Error during greeting decoding.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GreetingDecodeError {
    /// More data is needed.
    Incomplete,

    /// Decoding failed.
    Failed,
}

/// Error during command decoding.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandDecodeError<'a> {
    /// More data is needed.
    Incomplete,

    /// More data is needed (and further action may be necessary).
    ///
    /// The decoder stopped at the beginning of literal data. Typically, a server MUST send a
    /// command continuation request to agree to the receival of the remaining data. This behaviour
    /// is different when `LITERAL+/LITERAL-` is used.
    ///
    /// # With `LITERAL+/LITERAL-`
    ///
    /// When the `mode` is sync, everything is the same as above.
    ///
    /// When the `mode` is non-sync, *and* the server advertised the LITERAL+ capability,
    /// it MUST NOT send a command continuation request and accept the data right away.
    ///
    /// When the `mode` is non-sync, *and* the server advertised the LITERAL- capability,
    /// *and* the literal length is smaller or equal than 4096,
    /// it MUST NOT send a command continuation request and accept the data right away.
    ///
    /// When the `mode` is non-sync, *and* the server advertised the LITERAL- capability,
    /// *and* the literal length is greater than 4096,
    /// it MUST be handled as sync.
    ///
    /// ```rust,ignore
    /// match mode {
    ///     LiteralMode::Sync => /* Same as sync. */
    ///     LiteralMode::Sync => match advertised {
    ///         Capability::LiteralPlus => /* Accept data right away. */
    ///         Capability::LiteralMinus => {
    ///             if literal_length <= 4096 {
    ///                 /* Accept data right away. */
    ///             } else {
    ///                 /* Same as sync. */
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    LiteralFound {
        /// The corresponding command (tag) to which this literal is bound.
        ///
        /// This is required to reject literals, e.g., when their size exceeds a limit.
        tag: Tag<'a>,

        /// Literal length.
        length: u32,

        /// Literal mode, i.e., sync or non-sync.
        mode: LiteralMode,
    },

    /// Decoding failed.
    Failed,
}

/// Error during authenticate data line decoding.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthenticateDataDecodeError {
    /// More data is needed.
    Incomplete,

    /// Decoding failed.
    Failed,
}

/// Error during response decoding.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponseDecodeError {
    /// More data is needed.
    Incomplete,

    /// The decoder stopped at the beginning of literal data.
    ///
    /// The client *MUST* accept the literal and has no option to reject it.
    /// However, when the client ultimately does not want to handle the literal, it can do something
    /// similar to <https://datatracker.ietf.org/doc/html/rfc7888#section-4>.
    ///
    /// It can implement a discarding mechanism, basically, consuming the whole literal but not
    /// saving the bytes in memory. Or, it can close the connection.
    LiteralFound {
        /// Literal length.
        length: u32,
    },

    /// Decoding failed.
    Failed,
}

/// Error during idle done decoding.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdleDoneDecodeError {
    /// More data is needed.
    Incomplete,

    /// Decoding failed.
    Failed,
}

// -------------------------------------------------------------------------------------------------

impl Decoder for GreetingCodec {
    type Message<'a> = Greeting<'a>;
    type Error<'a> = GreetingDecodeError;

    fn decode<'a>(
        &self,
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self::Message<'a>), Self::Error<'static>> {
        match greeting(input) {
            Ok((rem, grt)) => Ok((rem, grt)),
            Err(nom::Err::Incomplete(_)) => Err(GreetingDecodeError::Incomplete),
            Err(nom::Err::Failure(_)) | Err(nom::Err::Error(_)) => Err(GreetingDecodeError::Failed),
        }
    }
}

impl Decoder for CommandCodec {
    type Message<'a> = Command<'a>;
    type Error<'a> = CommandDecodeError<'a>;

    fn decode<'a>(
        &self,
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self::Message<'a>), Self::Error<'a>> {
        match command(input) {
            Ok((rem, cmd)) => Ok((rem, cmd)),
            Err(nom::Err::Incomplete(_)) => Err(CommandDecodeError::Incomplete),
            Err(nom::Err::Failure(error)) => match error {
                IMAPParseError {
                    input: _,
                    kind: IMAPErrorKind::Literal { tag, length, mode },
                } => Err(CommandDecodeError::LiteralFound {
                    // Unwrap: We *must* receive a `tag` during command parsing.
                    tag: tag.expect("Expected `Some(tag)` in `IMAPErrorKind::Literal`, got `None`"),
                    length,
                    mode,
                }),
                _ => Err(CommandDecodeError::Failed),
            },
            Err(nom::Err::Error(_)) => Err(CommandDecodeError::Failed),
        }
    }
}

impl Decoder for ResponseCodec {
    type Message<'a> = Response<'a>;
    type Error<'a> = ResponseDecodeError;

    fn decode<'a>(
        &self,
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self::Message<'a>), Self::Error<'static>> {
        match response(input) {
            Ok((rem, rsp)) => Ok((rem, rsp)),
            Err(nom::Err::Incomplete(_)) => Err(ResponseDecodeError::Incomplete),
            Err(nom::Err::Error(error) | nom::Err::Failure(error)) => match error {
                IMAPParseError {
                    kind: IMAPErrorKind::Literal { length, .. },
                    ..
                } => Err(ResponseDecodeError::LiteralFound { length }),
                _ => Err(ResponseDecodeError::Failed),
            },
        }
    }
}

impl Decoder for AuthenticateDataCodec {
    type Message<'a> = AuthenticateData;
    type Error<'a> = AuthenticateDataDecodeError;

    fn decode<'a>(
        &self,
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self::Message<'a>), Self::Error<'static>> {
        match authenticate_data(input) {
            Ok((rem, rsp)) => Ok((rem, rsp)),
            Err(nom::Err::Incomplete(_)) => Err(AuthenticateDataDecodeError::Incomplete),
            Err(nom::Err::Failure(_)) | Err(nom::Err::Error(_)) => {
                Err(AuthenticateDataDecodeError::Failed)
            }
        }
    }
}

impl Decoder for IdleDoneCodec {
    type Message<'a> = IdleDone;
    type Error<'a> = IdleDoneDecodeError;

    fn decode<'a>(
        &self,
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self::Message<'a>), Self::Error<'static>> {
        match idle_done(input) {
            Ok((rem, rsp)) => Ok((rem, rsp)),
            Err(nom::Err::Incomplete(_)) => Err(IdleDoneDecodeError::Incomplete),
            Err(nom::Err::Failure(_)) | Err(nom::Err::Error(_)) => Err(IdleDoneDecodeError::Failed),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use imap_types::{
        command::{Command, CommandBody},
        core::{IString, Literal, NString, NonEmptyVec},
        extensions::idle::IdleDone,
        fetch::MessageDataItem,
        mailbox::Mailbox,
        response::{Data, Greeting, GreetingKind, Response},
        secret::Secret,
    };

    use super::*;

    #[test]
    fn test_decode_greeting() {
        let tests = [
            // Ok
            (
                b"* OK ...\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    Greeting::new(GreetingKind::Ok, None, "...").unwrap(),
                )),
            ),
            (
                b"* ByE .\r\n???".as_ref(),
                Ok((
                    b"???".as_ref(),
                    Greeting::new(GreetingKind::Bye, None, ".").unwrap(),
                )),
            ),
            (
                b"* preaUth x\r\n?".as_ref(),
                Ok((
                    b"?".as_ref(),
                    Greeting::new(GreetingKind::PreAuth, None, "x").unwrap(),
                )),
            ),
            // Incomplete
            (b"*".as_ref(), Err(GreetingDecodeError::Incomplete)),
            (b"* ".as_ref(), Err(GreetingDecodeError::Incomplete)),
            (b"* O".as_ref(), Err(GreetingDecodeError::Incomplete)),
            (b"* OK".as_ref(), Err(GreetingDecodeError::Incomplete)),
            (b"* OK ".as_ref(), Err(GreetingDecodeError::Incomplete)),
            (b"* OK .".as_ref(), Err(GreetingDecodeError::Incomplete)),
            (b"* OK .\r".as_ref(), Err(GreetingDecodeError::Incomplete)),
            // Failed
            (b"**".as_ref(), Err(GreetingDecodeError::Failed)),
            (b"* NO x\r\n".as_ref(), Err(GreetingDecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = GreetingCodec::default().decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = GreetingCodec::default().decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }

    #[test]
    fn test_decode_command() {
        let tests = [
            // Ok
            (
                b"a noop\r\n".as_ref(),
                Ok((b"".as_ref(), Command::new("a", CommandBody::Noop).unwrap())),
            ),
            (
                b"a noop\r\n???".as_ref(),
                Ok((
                    b"???".as_ref(),
                    Command::new("a", CommandBody::Noop).unwrap(),
                )),
            ),
            (
                b"a select {5}\r\ninbox\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    Command::new(
                        "a",
                        CommandBody::Select {
                            mailbox: Mailbox::Inbox,
                            parameters: None,
                        },
                    )
                    .unwrap(),
                )),
            ),
            (
                b"a select {5}\r\ninbox\r\nxxx".as_ref(),
                Ok((
                    b"xxx".as_ref(),
                    Command::new(
                        "a",
                        CommandBody::Select {
                            mailbox: Mailbox::Inbox,
                            parameters: None,
                        },
                    )
                    .unwrap(),
                )),
            ),
            // Incomplete
            (b"a".as_ref(), Err(CommandDecodeError::Incomplete)),
            (b"a ".as_ref(), Err(CommandDecodeError::Incomplete)),
            (b"a n".as_ref(), Err(CommandDecodeError::Incomplete)),
            (b"a no".as_ref(), Err(CommandDecodeError::Incomplete)),
            (b"a noo".as_ref(), Err(CommandDecodeError::Incomplete)),
            (b"a noop".as_ref(), Err(CommandDecodeError::Incomplete)),
            (b"a noop\r".as_ref(), Err(CommandDecodeError::Incomplete)),
            // LiteralAckRequired
            (
                b"a select {5}\r\n".as_ref(),
                Err(CommandDecodeError::LiteralFound {
                    tag: Tag::try_from("a").unwrap(),
                    length: 5,
                    mode: LiteralMode::Sync,
                }),
            ),
            // Incomplete (after literal)
            (
                b"a select {5}\r\nxxx".as_ref(),
                Err(CommandDecodeError::Incomplete),
            ),
            // Failed
            (b"* noop\r\n".as_ref(), Err(CommandDecodeError::Failed)),
            (b"A  noop\r\n".as_ref(), Err(CommandDecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = CommandCodec::default().decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = CommandCodec::default().decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }

    #[test]
    fn test_decode_authenticate_data() {
        let tests = [
            // Ok
            (
                b"VGVzdA==\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    AuthenticateData::Continue(Secret::new(b"Test".to_vec())),
                )),
            ),
            (
                b"VGVzdA==\r\nx".as_ref(),
                Ok((
                    b"x".as_ref(),
                    AuthenticateData::Continue(Secret::new(b"Test".to_vec())),
                )),
            ),
            (
                b"*\r\n".as_ref(),
                Ok((b"".as_ref(), AuthenticateData::Cancel)),
            ),
            (
                b"*\r\nx".as_ref(),
                Ok((b"x".as_ref(), AuthenticateData::Cancel)),
            ),
            // Incomplete
            (b"V".as_ref(), Err(AuthenticateDataDecodeError::Incomplete)),
            (b"VG".as_ref(), Err(AuthenticateDataDecodeError::Incomplete)),
            (
                b"VGV".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVz".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVzd".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVzdA".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVzdA=".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVzdA==".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVzdA==\r".as_ref(),
                Err(AuthenticateDataDecodeError::Incomplete),
            ),
            (
                b"VGVzdA==\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    AuthenticateData::Continue(Secret::new(b"Test".to_vec())),
                )),
            ),
            // Failed
            (
                b"VGVzdA== \r\n".as_ref(),
                Err(AuthenticateDataDecodeError::Failed),
            ),
            (
                b" VGVzdA== \r\n".as_ref(),
                Err(AuthenticateDataDecodeError::Failed),
            ),
            (
                b" V GVzdA== \r\n".as_ref(),
                Err(AuthenticateDataDecodeError::Failed),
            ),
            (
                b" V GVzdA= \r\n".as_ref(),
                Err(AuthenticateDataDecodeError::Failed),
            ),
        ];

        for (test, expected) in tests {
            let got = AuthenticateDataCodec::default().decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = AuthenticateDataCodec::default().decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }

    #[test]
    fn test_decode_idle_done() {
        let tests = [
            // Ok
            (b"done\r\n".as_ref(), Ok((b"".as_ref(), IdleDone))),
            (b"done\r\n?".as_ref(), Ok((b"?".as_ref(), IdleDone))),
            // Incomplete
            (b"d".as_ref(), Err(IdleDoneDecodeError::Incomplete)),
            (b"do".as_ref(), Err(IdleDoneDecodeError::Incomplete)),
            (b"don".as_ref(), Err(IdleDoneDecodeError::Incomplete)),
            (b"done".as_ref(), Err(IdleDoneDecodeError::Incomplete)),
            (b"done\r".as_ref(), Err(IdleDoneDecodeError::Incomplete)),
            // Failed
            (b"donee\r\n".as_ref(), Err(IdleDoneDecodeError::Failed)),
            (b" done\r\n".as_ref(), Err(IdleDoneDecodeError::Failed)),
            (b"done \r\n".as_ref(), Err(IdleDoneDecodeError::Failed)),
            (b" done \r\n".as_ref(), Err(IdleDoneDecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = IdleDoneCodec::default().decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = IdleDoneCodec::default().decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }

    #[test]
    fn test_decode_response() {
        let tests = [
            // Incomplete
            (b"".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"*".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* ".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* S".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SE".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SEA".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SEAR".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SEARC".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SEARCH".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SEARCH ".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (b"* SEARCH 1".as_ref(), Err(ResponseDecodeError::Incomplete)),
            (
                b"* SEARCH 1\r".as_ref(),
                Err(ResponseDecodeError::Incomplete),
            ),
            // Ok
            (
                b"* SEARCH 1\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    Response::Data(Data::Search(vec![NonZeroU32::new(1).unwrap()])),
                )),
            ),
            (
                b"* SEARCH 1\r\n???".as_ref(),
                Ok((
                    b"???".as_ref(),
                    Response::Data(Data::Search(vec![NonZeroU32::new(1).unwrap()])),
                )),
            ),
            (
                b"* 1 FETCH (RFC822 {5}\r\nhello)\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    Response::Data(Data::Fetch {
                        seq: NonZeroU32::new(1).unwrap(),
                        items: NonEmptyVec::from(MessageDataItem::Rfc822(NString(Some(
                            IString::Literal(Literal::try_from(b"hello".as_ref()).unwrap()),
                        )))),
                    }),
                )),
            ),
            (
                b"* 1 FETCH (RFC822 {5}\r\n".as_ref(),
                Err(ResponseDecodeError::LiteralFound { length: 5 }),
            ),
            // Failed
            (
                b"*  search 1 2 3\r\n".as_ref(),
                Err(ResponseDecodeError::Failed),
            ),
            (b"A search\r\n".as_ref(), Err(ResponseDecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = ResponseCodec::default().decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = ResponseCodec::default().decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }
}
