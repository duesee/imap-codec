use std::io::{Error as IoError, Write};

use bounded_static::IntoBoundedStatic;
use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use super::{find_crlf_inclusive, parse_literal, LineError, LiteralError, LiteralFramingState};
use crate::{
    codec::{Decode, Encode},
    command::Command,
    response::{Greeting, Response},
    state::State as ImapState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapClientCodec {
    state: LiteralFramingState,
    imap_state: ImapState<'static>,
    max_literal_length: u32,
}

impl ImapClientCodec {
    pub fn new(max_literal_length: u32) -> Self {
        Self {
            state: LiteralFramingState::ReadLine { to_consume_acc: 0 },
            imap_state: ImapState::Greeting,
            max_literal_length,
        }
    }
}

#[derive(Debug, Error)]
pub enum ImapClientCodecError {
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    Line(#[from] LineError),
    #[error(transparent)]
    Literal(#[from] LiteralError),
    #[error("Parsing failed")]
    ResponseParsingFailed,
}

impl PartialEq for ImapClientCodecError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(error1), Self::Io(error2)) => error1.kind() == error2.kind(),
            (Self::Line(kind2), Self::Line(kind1)) => kind1 == kind2,
            (Self::Literal(kind1), Self::Literal(kind2)) => kind1 == kind2,
            (Self::ResponseParsingFailed, Self::ResponseParsingFailed) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Greeting(Greeting<'static>),
    Response(Response<'static>),
}

impl Decoder for ImapClientCodec {
    type Item = Event;
    type Error = ImapClientCodecError;

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
                                Ok(None) => {
                                    let parser = match self.imap_state {
                                        ImapState::Greeting => |input| {
                                            Greeting::decode(input).map(|(rem, grt)| {
                                                (rem, Event::Greeting(grt.into_static()))
                                            })
                                        },
                                        _ => |input| {
                                            Response::decode(input).map(|(rem, rsp)| {
                                                (rem, Event::Response(rsp.into_static()))
                                            })
                                        },
                                    };

                                    match parser(&src[..*to_consume_acc]) {
                                        Ok((rem, outcome)) => {
                                            assert!(rem.is_empty());

                                            src.advance(*to_consume_acc);
                                            self.state =
                                                LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                            if self.imap_state == ImapState::Greeting {
                                                // TODO: use other states, too? Why?
                                                self.imap_state = ImapState::NotAuthenticated;
                                            }

                                            return Ok(Some(outcome));
                                        }
                                        Err(_) => {
                                            src.advance(*to_consume_acc);

                                            return Err(
                                                ImapClientCodecError::ResponseParsingFailed,
                                            );
                                        }
                                    }
                                }
                                // Literal found.
                                Ok(Some(length)) => {
                                    if self.max_literal_length < length {
                                        src.advance(*to_consume_acc);
                                        self.state =
                                            LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                        // TODO: What should the client do?
                                        return Err(ImapClientCodecError::Literal(
                                            LiteralError::TooLarge {
                                                max_length: self.max_literal_length,
                                                length,
                                            },
                                        ));
                                    }

                                    src.reserve(length as usize);

                                    self.state = LiteralFramingState::ReadLiteral {
                                        to_consume_acc: *to_consume_acc,
                                        length,
                                    };
                                }
                                // Error processing literal.
                                Err(error) => {
                                    src.clear();
                                    self.state =
                                        LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                    return Err(ImapClientCodecError::Literal(error));
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

                            return Err(ImapClientCodecError::Line(error));
                        }
                    }
                }
                LiteralFramingState::ReadLiteral {
                    to_consume_acc,
                    length,
                } => {
                    if to_consume_acc + length as usize <= src.len() {
                        self.state = LiteralFramingState::ReadLine {
                            to_consume_acc: to_consume_acc + length as usize,
                        }
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }
}

impl<'a> Encoder<&Command<'a>> for ImapClientCodec {
    type Error = IoError;

    fn encode(&mut self, item: &Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        //dst.reserve(item.len());
        let mut writer = dst.writer();
        // TODO(225): Don't use `dump` here.
        let data = item.encode().dump();
        writer.write_all(&data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_util::codec::Decoder;

    use super::*;
    use crate::{
        core::{Literal, NString},
        fetch::FetchAttributeValue,
        response::{Data, GreetingKind},
        section::Section,
    };

    #[test]
    fn test_decoder_line() {
        let tests = [
            (b"".as_ref(), Ok(None)),
            (b"* ", Ok(None)),
            (b"OK ...\r", Ok(None)),
            (
                b"\n",
                Ok(Some(Event::Greeting(
                    Greeting::new(GreetingKind::Ok, None, "...").unwrap(),
                ))),
            ),
            (b"", Ok(None)),
            (b"xxxx", Ok(None)),
            (b"\r\n", Err(ImapClientCodecError::ResponseParsingFailed)),
        ];

        let mut src = BytesMut::new();
        let mut codec = ImapClientCodec::new(1024);

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
            (
                b"* OK ...\r\n".as_ref(),
                Ok(Some(Event::Greeting(
                    Greeting::new(GreetingKind::Ok, None, "...").unwrap(),
                ))),
            ),
            (b"* 12 FETCH (BODY[HEADER] {3}", Ok(None)),
            (b"\r", Ok(None)),
            (b"\n", Ok(None)),
            (b"a", Ok(None)),
            (b"bc)", Ok(None)),
            (b"\r", Ok(None)),
            (
                b"\n",
                Ok(Some(Event::Response(Response::Data(
                    Data::fetch(
                        12,
                        vec![FetchAttributeValue::BodyExt {
                            section: Some(Section::Header(None)),
                            origin: None,
                            data: NString(Some(Literal::try_from("abc").unwrap().into())),
                        }],
                    )
                    .unwrap(),
                )))),
            ),
        ];

        let mut src = BytesMut::new();
        let mut codec = ImapClientCodec::new(1024);

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
            // We still need to process the greeting first.
            (
                b"* OK ...\r\n".as_ref(),
                Ok(Some(Event::Greeting(
                    Greeting::new(GreetingKind::Ok, None, "...").unwrap(),
                ))),
            ),
            (
                b"xxx\r\n".as_ref(),
                Err(ImapClientCodecError::ResponseParsingFailed),
            ),
            (
                b"* search 1\n",
                Err(ImapClientCodecError::Line(LineError::NotCrLf)),
            ),
            (
                b"* 1 fetch (BODY[] {17}\r\naaaaaaaaaaaaaaaa)\r\n",
                Err(ImapClientCodecError::Literal(LiteralError::TooLarge {
                    max_length: 16,
                    length: 17,
                })),
            ),
        ];

        let mut src = BytesMut::new();
        let mut codec = ImapClientCodec::new(16);

        for (test, expected) in tests {
            src.extend_from_slice(test);
            let got = codec.decode(&mut src);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
        }
    }
}
