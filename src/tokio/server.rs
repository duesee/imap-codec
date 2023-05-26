use std::io::Error;

use bytes::{Buf, BufMut, BytesMut};
use imap_types::{bounded_static::IntoBoundedStatic, response::Greeting};
use tokio_util::codec::{Decoder, Encoder};

use super::{find_crlf_inclusive, parse_literal, LineError, LiteralError, LiteralFramingState};
use crate::{
    codec::{Decode, Encode},
    command::Command,
    response::Response,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapServerCodec {
    state: LiteralFramingState,
    max_literal_size: usize,
}

impl ImapServerCodec {
    pub fn new(max_literal_size: usize) -> Self {
        Self {
            state: LiteralFramingState::ReadLine { to_consume_acc: 0 },
            max_literal_size,
        }
    }
}

#[derive(Debug)]
pub enum ImapServerCodecError {
    Io(Error),
    Line(LineError),
    Literal(LiteralError),
    CommandParsingFailed,
    ActionRequired,
}

impl PartialEq for ImapServerCodecError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(error1), Self::Io(error2)) => error1.kind() == error2.kind(),
            (Self::Line(kind1), Self::Line(kind2)) => kind1 == kind2,
            (Self::Literal(kind1), Self::Literal(kind2)) => kind1 == kind2,
            (Self::CommandParsingFailed, Self::CommandParsingFailed) => true,
            (Self::ActionRequired, Self::ActionRequired) => true,
            _ => false,
        }
    }
}

impl From<Error> for ImapServerCodecError {
    fn from(error: Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OutcomeServer {
    Command(Command<'static>),
    ActionRequired(Action),
    // More might be require.
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    SendLiteralAck(u32),
    SendLiteralReject(u32),
}

impl Decoder for ImapServerCodec {
    type Item = OutcomeServer;
    type Error = ImapServerCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                LiteralFramingState::ReadLine {
                    ref mut to_consume_acc,
                } => {
                    match find_crlf_inclusive(*to_consume_acc, src) {
                        Ok(Some(to_consume)) => {
                            *to_consume_acc += to_consume;

                            match parse_literal(&src[..*to_consume_acc - 2]) {
                                // No literal.
                                Ok(None) => match Command::decode(&src[..*to_consume_acc]) {
                                    Ok((rem, cmd)) => {
                                        assert!(rem.is_empty());
                                        let cmd = cmd.into_static();

                                        src.advance(*to_consume_acc);
                                        self.state =
                                            LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                        return Ok(Some(OutcomeServer::Command(cmd)));
                                    }
                                    Err(_) => {
                                        src.advance(*to_consume_acc);

                                        return Err(ImapServerCodecError::CommandParsingFailed);
                                    }
                                },
                                // Literal found.
                                Ok(Some(needed)) => {
                                    if self.max_literal_size < needed as usize {
                                        src.advance(*to_consume_acc);
                                        self.state =
                                            LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                        return Ok(Some(OutcomeServer::ActionRequired(
                                            Action::SendLiteralReject(needed),
                                        )));
                                    }

                                    src.reserve(needed as usize);

                                    self.state = LiteralFramingState::ReadLiteral {
                                        to_consume_acc: *to_consume_acc,
                                        needed,
                                    };

                                    return Ok(Some(OutcomeServer::ActionRequired(
                                        Action::SendLiteralAck(needed),
                                    )));
                                }
                                // Error processing literal.
                                Err(error) => {
                                    src.clear();
                                    self.state =
                                        LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                    return Err(ImapServerCodecError::Literal(error));
                                }
                            }
                        }
                        // More data needed.
                        Ok(None) => {
                            return Ok(None);
                        }
                        // Error processing newline.
                        Err(error) => {
                            src.clear();
                            self.state = LiteralFramingState::ReadLine { to_consume_acc: 0 };

                            return Err(ImapServerCodecError::Line(error));
                        }
                    }
                }
                LiteralFramingState::ReadLiteral {
                    to_consume_acc,
                    needed,
                } => {
                    if to_consume_acc + needed as usize <= src.len() {
                        self.state = LiteralFramingState::ReadLine {
                            to_consume_acc: to_consume_acc + needed as usize,
                        }
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }
}

impl<'a> Encoder<&Greeting<'a>> for ImapServerCodec {
    type Error = Error;

    fn encode(&mut self, item: &Greeting, dst: &mut BytesMut) -> Result<(), Error> {
        //dst.reserve(item.len());
        let mut writer = dst.writer();
        item.encode(&mut writer).unwrap();
        Ok(())
    }
}

impl<'a> Encoder<&Response<'a>> for ImapServerCodec {
    type Error = Error;

    fn encode(&mut self, item: &Response, dst: &mut BytesMut) -> Result<(), Error> {
        //dst.reserve(item.len());
        let mut writer = dst.writer();
        item.encode(&mut writer).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use imap_types::{
        command::{Command, CommandBody},
        core::{AString, AtomExt, IString, Literal},
        secret::Secret,
    };
    use tokio_util::codec::Decoder;

    use super::*;

    #[test]
    fn test_decoder_line() {
        let tests = [
            (b"".as_ref(), Ok(None)),
            (b"a noop", Ok(None)),
            (b"\r", Ok(None)),
            (
                b"\n",
                Ok(Some(OutcomeServer::Command(
                    Command::new("a", CommandBody::Noop).unwrap(),
                ))),
            ),
            (b"", Ok(None)),
            (b"xxxx", Ok(None)),
            (b"\r\n", Err(ImapServerCodecError::CommandParsingFailed)),
        ];

        let mut src = BytesMut::new();
        let mut codec = ImapServerCodec::new(1024);

        for (test, expected) in tests {
            src.extend_from_slice(test);
            let got = codec.decode(&mut src);

            assert_eq!(expected, got);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
        }
    }

    #[test]
    fn test_decoder_literal() {
        let tests = [
            (b"".as_ref(), Ok(None)),
            (b"a login", Ok(None)),
            (b" {", Ok(None)),
            (b"5", Ok(None)),
            (b"}", Ok(None)),
            (
                b"\r\n",
                Ok(Some(OutcomeServer::ActionRequired(Action::SendLiteralAck(
                    5,
                )))),
            ),
            (b"a", Ok(None)),
            (b"l", Ok(None)),
            (b"i", Ok(None)),
            (b"ce", Ok(None)),
            (b" ", Ok(None)),
            (
                b"password\r\n",
                Ok(Some(OutcomeServer::Command(
                    Command::new(
                        "a",
                        CommandBody::Login {
                            username: AString::String(IString::Literal(
                                Literal::try_from(b"alice".as_ref()).unwrap(),
                            )),
                            password: Secret::new(AString::Atom(
                                AtomExt::try_from("password").unwrap(),
                            )),
                        },
                    )
                    .unwrap(),
                ))),
            ),
        ];

        let mut src = BytesMut::new();
        let mut codec = ImapServerCodec::new(1024);

        for (test, expected) in tests {
            src.extend_from_slice(test);
            let got = codec.decode(&mut src);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_decoder_error() {
        let tests = [
            (
                b"xxx\r\n".as_ref(),
                Err(ImapServerCodecError::CommandParsingFailed),
            ),
            (
                b"a noop\n",
                Err(ImapServerCodecError::Line(LineError::NotCrLf)),
            ),
            (
                b"a login alice {16}\r\n",
                Ok(Some(OutcomeServer::ActionRequired(Action::SendLiteralAck(
                    16,
                )))),
            ),
            (
                b"aaaaaaaaaaaaaaaa\r\n",
                Ok(Some(OutcomeServer::Command(
                    Command::new(
                        "a",
                        CommandBody::login("alice", Literal::try_from("aaaaaaaaaaaaaaaa").unwrap())
                            .unwrap(),
                    )
                    .unwrap(),
                ))),
            ),
            (
                b"a login alice {17}\r\n",
                Ok(Some(OutcomeServer::ActionRequired(
                    Action::SendLiteralReject(17),
                ))),
            ),
            (
                b"a login alice {1-}\r\n",
                Err(ImapServerCodecError::Literal(LiteralError::BadNumber)),
            ),
            (
                b"a login alice }\r\n",
                Err(ImapServerCodecError::Literal(LiteralError::NoOpeningBrace)),
            ),
        ];

        let mut src = BytesMut::new();
        let mut codec = ImapServerCodec::new(16);

        for (test, expected) in tests {
            src.extend_from_slice(test);
            let got = codec.decode(&mut src);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
        }
    }
}
