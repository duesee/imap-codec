use std::io::Error;

use bytes::{Buf, BufMut, BytesMut};
use imap_types::bounded_static::IntoBoundedStatic;
use tokio_util::codec::{Decoder, Encoder};

use super::{find_crlf_inclusive, parse_literal, LineKind, LiteralKind, State};
use crate::{
    codec::Decode,
    types::{codec::Encode, command::Command, response::Response},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapServerCodec {
    state: State,
    max_literal_size: usize,
}

impl ImapServerCodec {
    pub fn new(max_literal_size: usize) -> Self {
        Self {
            state: State::ReadLine { to_consume_acc: 0 },
            max_literal_size,
        }
    }
}

#[derive(Debug)]
pub enum ImapServerCodecError {
    Io(std::io::Error),
    Line(LineKind),
    Literal(LiteralKind),
    CommandParsingFailed,
    ActionRequired,
}

impl PartialEq for ImapServerCodecError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(error1), Self::Io(error2)) => error1.kind() == error2.kind(),
            (Self::Line(kind2), Self::Line(kind1)) => kind1 == kind2,
            (Self::Literal(kind1), Self::Literal(kind2)) => kind1 == kind2,
            (Self::CommandParsingFailed, Self::CommandParsingFailed) => true,
            (Self::ActionRequired, Self::ActionRequired) => true,
            _ => false,
        }
    }
}

impl From<std::io::Error> for ImapServerCodecError {
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
                State::ReadLine {
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
                                        self.state = State::ReadLine { to_consume_acc: 0 };

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
                                        self.state = State::ReadLine { to_consume_acc: 0 };

                                        return Ok(Some(OutcomeServer::ActionRequired(
                                            Action::SendLiteralReject(needed),
                                        )));
                                    }

                                    src.reserve(needed as usize);

                                    self.state = State::ReadLiteral {
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
                                    self.state = State::ReadLine { to_consume_acc: 0 };

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
                            self.state = State::ReadLine { to_consume_acc: 0 };

                            return Err(ImapServerCodecError::Line(error));
                        }
                    }
                }
                State::ReadLiteral {
                    to_consume_acc,
                    needed,
                } => {
                    if to_consume_acc + needed as usize <= src.len() {
                        self.state = State::ReadLine {
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

impl<'a> Encoder<&Response<'a>> for ImapServerCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Response, dst: &mut BytesMut) -> Result<(), std::io::Error> {
        //dst.reserve(item.len());
        let mut writer = dst.writer();
        item.encode(&mut writer).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use bytes::BytesMut;
    use imap_types::{
        command::{Command, CommandBody},
        core::{AString, AtomExt, IString, Literal},
    };
    use tokio_util::codec::Decoder;

    use super::{Action, ImapServerCodec, ImapServerCodecError, OutcomeServer};

    #[test]
    fn decoder_line() {
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
    fn decoder_literal() {
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
                            password: AString::Atom(AtomExt::try_from("password").unwrap()),
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
    fn decoder_error() {
        let tests = [
            (
                b"xxx\r\n".as_ref(),
                Err(ImapServerCodecError::CommandParsingFailed),
            ),
            (
                b"a noop\r\n",
                Ok(Some(OutcomeServer::Command(
                    Command::new("a", CommandBody::Noop).unwrap(),
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
}
