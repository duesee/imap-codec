use std::num::{ParseIntError, TryFromIntError};

#[cfg(feature = "bounded-static")]
use bounded_static::IntoBoundedStatic;
#[cfg(feature = "ext_literal")]
use imap_types::core::LiteralMode;
#[cfg(feature = "ext_idle")]
use imap_types::extensions::idle::IdleDone;
use imap_types::{
    auth::AuthenticateData,
    command::Command,
    response::{Continue, Greeting, Response},
};
use nom::error::{ErrorKind, FromExternalError, ParseError};

#[cfg(feature = "ext_idle")]
use crate::codec::IdleDoneCodec;
#[cfg(feature = "ext_idle")]
use crate::extensions::idle::idle_done;
use crate::{
    auth::authenticate_data,
    codec::{AuthenticateDataCodec, CommandCodec, ContinueCodec, GreetingCodec, ResponseCodec},
    command::command,
    response::{continue_req, greeting, response},
};

/// An extended version of [`nom::IResult`].
pub(crate) type IMAPResult<I, O> = Result<(I, O), nom::Err<IMAPParseError<I>>>;

/// An extended version of [`nom::error::Error`].
#[derive(Debug)]
pub(crate) struct IMAPParseError<I> {
    #[allow(unused)]
    pub input: I,
    pub kind: IMAPErrorKind,
}

/// An extended version of [`nom::error::ErrorKind`].
#[derive(Debug)]
pub(crate) enum IMAPErrorKind {
    Literal {
        length: u32,
        #[cfg(feature = "ext_literal")]
        mode: LiteralMode,
    },
    BadNumber,
    BadBase64,
    BadDateTime,
    LiteralContainsNull,
    RecursionLimitExceeded,
    Nom(ErrorKind),
}

impl<I> ParseError<I> for IMAPParseError<I> {
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

impl<I> FromExternalError<I, ParseIntError> for IMAPParseError<I> {
    fn from_external_error(input: I, _: ErrorKind, _: ParseIntError) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::BadNumber,
        }
    }
}

impl<I> FromExternalError<I, TryFromIntError> for IMAPParseError<I> {
    fn from_external_error(input: I, _: ErrorKind, _: TryFromIntError) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::BadNumber,
        }
    }
}

impl<I> FromExternalError<I, base64::DecodeError> for IMAPParseError<I> {
    fn from_external_error(input: I, _: ErrorKind, _: base64::DecodeError) -> Self {
        Self {
            input,
            kind: IMAPErrorKind::BadBase64,
        }
    }
}

pub trait Decoder {
    type Item<'a>: Sized;

    fn decode(input: &[u8]) -> Result<(&[u8], Self::Item<'_>), DecodeError>;

    #[cfg(feature = "bounded-static")]
    fn decode_static<'a>(input: &'a [u8]) -> Result<(&'a [u8], Self::Item<'static>), DecodeError>
    where
        Self::Item<'a>: IntoBoundedStatic<Static = Self::Item<'static>>,
    {
        let (remaining, value) = Self::decode(input)?;
        Ok((remaining, value.into_static()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// More data is needed.
    Incomplete,

    /// More data is needed (and further action may be necessary).
    ///
    /// The decoder stopped at the beginning of literal data. When in the role of a server, sending
    /// a continuation request may be necessary to agree to the receival of the remaining data.
    LiteralFound {
        length: u32,
        #[cfg(feature = "ext_literal")]
        mode: LiteralMode,
    },

    // Decoding failed.
    Failed,
}

// -------------------------------------------------------------------------------------------------

macro_rules! impl_decode_for_object {
    ($decoder:ident, $item:ty, $parser:ident) => {
        impl Decoder for $decoder {
            type Item<'a> = $item;

            fn decode(input: &[u8]) -> Result<(&[u8], Self::Item<'_>), DecodeError> {
                match $parser(input) {
                    Ok((rem, grt)) => Ok((rem, grt)),
                    Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
                    Err(nom::Err::Failure(error)) => match error {
                        IMAPParseError {
                            kind:
                                IMAPErrorKind::Literal {
                                    length,
                                    #[cfg(feature = "ext_literal")]
                                    mode,
                                },
                            ..
                        } => Err(DecodeError::LiteralFound {
                            length,
                            #[cfg(feature = "ext_literal")]
                            mode,
                        }),
                        _ => Err(DecodeError::Failed),
                    },
                    Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
                }
            }
        }
    };
}

impl_decode_for_object!(GreetingCodec, Greeting<'a>, greeting);
impl_decode_for_object!(CommandCodec, Command<'a>, command);
impl_decode_for_object!(AuthenticateDataCodec, AuthenticateData, authenticate_data);
#[cfg(feature = "ext_idle")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_idle")))]
impl_decode_for_object!(IdleDoneCodec, IdleDone, idle_done);
impl_decode_for_object!(ResponseCodec, Response<'a>, response);
impl_decode_for_object!(ContinueCodec, Continue<'a>, continue_req);

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    #[cfg(feature = "ext_idle")]
    use imap_types::extensions::idle::IdleDone;
    use imap_types::{
        command::{Command, CommandBody},
        core::{IString, Literal, NString, NonEmptyVec},
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
            (b"*".as_ref(), Err(DecodeError::Incomplete)),
            (b"* ".as_ref(), Err(DecodeError::Incomplete)),
            (b"* O".as_ref(), Err(DecodeError::Incomplete)),
            (b"* OK".as_ref(), Err(DecodeError::Incomplete)),
            (b"* OK ".as_ref(), Err(DecodeError::Incomplete)),
            (b"* OK .".as_ref(), Err(DecodeError::Incomplete)),
            (b"* OK .\r".as_ref(), Err(DecodeError::Incomplete)),
            // Failed
            (b"**".as_ref(), Err(DecodeError::Failed)),
            (b"* NO x\r\n".as_ref(), Err(DecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = GreetingCodec::decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = GreetingCodec::decode_static(test);
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
                        },
                    )
                    .unwrap(),
                )),
            ),
            // Incomplete
            (b"a".as_ref(), Err(DecodeError::Incomplete)),
            (b"a ".as_ref(), Err(DecodeError::Incomplete)),
            (b"a n".as_ref(), Err(DecodeError::Incomplete)),
            (b"a no".as_ref(), Err(DecodeError::Incomplete)),
            (b"a noo".as_ref(), Err(DecodeError::Incomplete)),
            (b"a noop".as_ref(), Err(DecodeError::Incomplete)),
            (b"a noop\r".as_ref(), Err(DecodeError::Incomplete)),
            // LiteralAckRequired
            (
                b"a select {5}\r\n".as_ref(),
                Err(DecodeError::LiteralFound {
                    length: 5,

                    #[cfg(feature = "ext_literal")]
                    mode: LiteralMode::Sync,
                }),
            ),
            // Incomplete (after literal)
            (
                b"a select {5}\r\nxxx".as_ref(),
                Err(DecodeError::Incomplete),
            ),
            // Failed
            (b"* noop\r\n".as_ref(), Err(DecodeError::Failed)),
            (b"A  noop\r\n".as_ref(), Err(DecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = CommandCodec::decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = CommandCodec::decode_static(test);
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
                    AuthenticateData(Secret::new(b"Test".to_vec())),
                )),
            ),
            (
                b"VGVzdA==\r\nx".as_ref(),
                Ok((
                    b"x".as_ref(),
                    AuthenticateData(Secret::new(b"Test".to_vec())),
                )),
            ),
            // Incomplete
            (b"V".as_ref(), Err(DecodeError::Incomplete)),
            (b"VG".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGV".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGVz".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGVzd".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGVzdA".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGVzdA=".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGVzdA==".as_ref(), Err(DecodeError::Incomplete)),
            (b"VGVzdA==\r".as_ref(), Err(DecodeError::Incomplete)),
            (
                b"VGVzdA==\r\n".as_ref(),
                Ok((
                    b"".as_ref(),
                    AuthenticateData(Secret::new(b"Test".to_vec())),
                )),
            ),
            // Failed
            (b"VGVzdA== \r\n".as_ref(), Err(DecodeError::Failed)),
            (b" VGVzdA== \r\n".as_ref(), Err(DecodeError::Failed)),
            (b" V GVzdA== \r\n".as_ref(), Err(DecodeError::Failed)),
            (b" V GVzdA= \r\n".as_ref(), Err(DecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = AuthenticateDataCodec::decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = AuthenticateDataCodec::decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }

    #[cfg(feature = "ext_idle")]
    #[test]
    fn test_decode_idle_done() {
        let tests = [
            // Ok
            (b"done\r\n".as_ref(), Ok((b"".as_ref(), IdleDone))),
            (b"done\r\n?".as_ref(), Ok((b"?".as_ref(), IdleDone))),
            // Incomplete
            (b"d".as_ref(), Err(DecodeError::Incomplete)),
            (b"do".as_ref(), Err(DecodeError::Incomplete)),
            (b"don".as_ref(), Err(DecodeError::Incomplete)),
            (b"done".as_ref(), Err(DecodeError::Incomplete)),
            (b"done\r".as_ref(), Err(DecodeError::Incomplete)),
            // Failed
            (b"donee\r\n".as_ref(), Err(DecodeError::Failed)),
            (b" done\r\n".as_ref(), Err(DecodeError::Failed)),
            (b"done \r\n".as_ref(), Err(DecodeError::Failed)),
            (b" done \r\n".as_ref(), Err(DecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = IdleDoneCodec::decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = IdleDoneCodec::decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }

    #[test]
    fn test_decode_response() {
        let tests = [
            // Incomplete
            (b"".as_ref(), Err(DecodeError::Incomplete)),
            (b"*".as_ref(), Err(DecodeError::Incomplete)),
            (b"* ".as_ref(), Err(DecodeError::Incomplete)),
            (b"* S".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SE".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEA".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEAR".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEARC".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEARCH".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEARCH ".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEARCH 1".as_ref(), Err(DecodeError::Incomplete)),
            (b"* SEARCH 1\r".as_ref(), Err(DecodeError::Incomplete)),
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
            // LiteralAck treated the same as in response, even though it could be `Incomplete`.
            (
                b"* 1 FETCH (RFC822 {5}\r\n".as_ref(),
                Err(DecodeError::LiteralFound {
                    length: 5,
                    #[cfg(feature = "ext_literal")]
                    mode: LiteralMode::Sync,
                }),
            ),
            // Failed
            (b"*  search 1 2 3\r\n".as_ref(), Err(DecodeError::Failed)),
            (b"A search\r\n".as_ref(), Err(DecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = ResponseCodec::decode(test);
            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
            assert_eq!(expected, got);

            #[cfg(feature = "bounded-static")]
            {
                let got = ResponseCodec::decode_static(test);
                assert_eq!(expected, got);
            }
        }
    }
}
