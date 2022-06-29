use std::io::Error;

use bytes::{Buf, BufMut, BytesMut};
use imap_types::bounded_static::IntoBoundedStatic;
use tokio_util::codec::{Decoder, Encoder};

use super::{find_crlf_inclusive, parse_literal, LineError, LiteralError, LiteralFramingState};
use crate::{
    codec::Decode,
    types::{
        codec::Encode,
        command::Command,
        response::{Greeting, Response},
        state::State as ImapState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapClientCodec {
    state: LiteralFramingState,
    imap_state: ImapState<'static>,
    max_literal_size: usize,
}

impl ImapClientCodec {
    pub fn new(max_literal_size: usize) -> Self {
        Self {
            state: LiteralFramingState::ReadLine { to_consume_acc: 0 },
            imap_state: ImapState::Greeting,
            max_literal_size,
        }
    }
}

#[derive(Debug)]
pub enum ImapClientCodecError {
    Io(std::io::Error),
    Line(LineError),
    Literal(LiteralError),
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

impl From<std::io::Error> for ImapClientCodecError {
    fn from(error: Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OutcomeClient {
    Respone(Response<'static>),
    // More might be require.
}

impl Decoder for ImapClientCodec {
    type Item = OutcomeClient;
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
                                            Greeting::decode(input)
                                                .map(|(rem, grt)| (rem, Response::Greeting(grt)))
                                        },
                                        _ => Response::decode,
                                    };

                                    match parser(&src[..*to_consume_acc]) {
                                        Ok((rem, rsp)) => {
                                            assert!(rem.is_empty());
                                            let rsp = rsp.into_static();

                                            src.advance(*to_consume_acc);
                                            self.state =
                                                LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                            if self.imap_state == ImapState::Greeting {
                                                // TODO: use other states, too? Why?
                                                self.imap_state = ImapState::NotAuthenticated;
                                            }

                                            return Ok(Some(OutcomeClient::Respone(rsp)));
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
                                Ok(Some(needed)) => {
                                    if self.max_literal_size < needed as usize {
                                        src.advance(*to_consume_acc);
                                        self.state =
                                            LiteralFramingState::ReadLine { to_consume_acc: 0 };

                                        // TODO: What should the client do?
                                        return Err(ImapClientCodecError::Literal(
                                            LiteralError::TooLarge(needed),
                                        ));
                                    }

                                    src.reserve(needed as usize);

                                    self.state = LiteralFramingState::ReadLiteral {
                                        to_consume_acc: *to_consume_acc,
                                        needed,
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

impl<'a> Encoder<&Command<'a>> for ImapClientCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Command, dst: &mut BytesMut) -> Result<(), std::io::Error> {
        //dst.reserve(item.len());
        let mut writer = dst.writer();
        item.encode(&mut writer).unwrap();
        Ok(())
    }
}
