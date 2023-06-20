use std::io::{Read, Write};

use bytes::{Buf, BytesMut};
use log::error;

#[cfg(feature = "ext_literal")]
use crate::core::LiteralMode;
use crate::{
    codec::{DecodeError, DecodeStatic, Encode, Fragment},
    response::Continue,
    stream::{find_crlf_inclusive, FramingState},
};

/// Manages a `stream` and a `buffer` to "split-off" messages as requested.
///
/// This struct abstracts over IO and handles IMAP literals.
#[derive(Debug)]
pub struct Client<Stream> {
    state: FramingState,
    stream: Stream,
    buffer: BytesMut,
}

impl<S> Client<S>
where
    S: Read + Write,
{
    /// Create a new client.
    pub fn new(stream: S) -> Self {
        Self {
            state: FramingState::default(),
            stream,
            buffer: BytesMut::default(),
        }
    }

    /// Send an `Encode`able IMAP message to the server.
    ///
    /// Note: This method takes care of literal handling.
    // TODO: Better error handling.
    #[allow(clippy::result_unit_err)]
    pub fn send<Message>(&mut self, message: Message) -> Result<(), ()>
    where
        Message: Encode,
    {
        for fragment in message.encode() {
            match fragment {
                Fragment::Line { data } => self.send_raw(&data).map_err(|_| ())?,
                #[cfg(not(feature = "ext_literal"))]
                Fragment::Literal { data } => {
                    self.recv::<Continue>()?;

                    self.send_raw(&data)?;
                }
                #[cfg(feature = "ext_literal")]
                Fragment::Literal { data, mode } => {
                    match mode {
                        LiteralMode::Sync => {
                            // FIXME: We also need to receive other `Response`s here, collect them,
                            //        and return them if they were sent in the meantime.
                            self.recv::<Continue>()?;
                        }
                        LiteralMode::NonSync => {}
                    }

                    self.send_raw(&data).map_err(|_| ())?;
                }
            }
        }

        Ok(())
    }

    /// Receive a `Decode`able message from the server.
    ///
    /// It's required to specify the expected message. This method takes care of literal handling.
    // TODO: Better error handling.
    #[allow(clippy::result_unit_err)]
    pub fn recv<Message>(&mut self) -> Result<Message, ()>
    where
        Message: DecodeStatic,
    {
        loop {
            match self.state {
                FramingState::ReadLine {
                    ref mut to_consume_acc,
                } => {
                    match find_crlf_inclusive(*to_consume_acc, self.buffer.as_ref()) {
                        Some(line) => match line {
                            // After skipping `to_consume_acc` bytes, we need `to_consume` more
                            // bytes to form a full line (including the `\r\n`).
                            Ok(to_consume) => {
                                *to_consume_acc += to_consume;
                                let line = &self.buffer.as_ref()[..*to_consume_acc];

                                match Message::decode(line) {
                                    // We got a complete message.
                                    Ok((rem, message)) => {
                                        assert!(rem.is_empty());

                                        self.buffer.advance(*to_consume_acc);
                                        self.state = FramingState::ReadLine { to_consume_acc: 0 };

                                        return Ok(message);
                                    }
                                    Err(error) => match error {
                                        // We supposedly need more data ...
                                        //
                                        // This should not happen because a line that doesn't end
                                        // with a literal is always "complete" in IMAP.
                                        DecodeError::Incomplete => {
                                            let discarded = self.buffer.split_to(*to_consume_acc);
                                            error!("Unexpected `Incomplete`. discarded = {discarded:?}");
                                        }
                                        // We found a literal.
                                        DecodeError::LiteralFound { length, .. } => {
                                            self.buffer.reserve(length as usize);

                                            self.state = FramingState::ReadLiteral {
                                                to_consume_acc: *to_consume_acc,
                                                length,
                                            };
                                        }
                                        DecodeError::Failed => {
                                            let discarded = self.buffer.split_to(*to_consume_acc);
                                            self.state =
                                                FramingState::ReadLine { to_consume_acc: 0 };
                                            error!("Parsing failed. discarded = {discarded:?}");
                                        }
                                    },
                                }
                            }
                            // After skipping `to_consume_acc` bytes, we need `to_consume` more
                            // bytes to form a full line (including the `\n`).
                            //
                            // Note: This line is missing the `\r\n` and should be discarded.
                            Err(to_discard) => {
                                *to_consume_acc += to_discard;
                                let discarded = self.buffer.split_to(*to_consume_acc);
                                error!("Expected `\r\n`, got `\n`. discarded = {discarded:?}");

                                self.state = FramingState::ReadLine { to_consume_acc: 0 };
                            }
                        },
                        // More data needed.
                        None => {
                            self.recv_raw().map_err(|_| ())?;
                        }
                    }
                }
                FramingState::ReadLiteral {
                    to_consume_acc,
                    length,
                } => {
                    if to_consume_acc + length as usize <= self.buffer.len() {
                        self.state = FramingState::ReadLine {
                            to_consume_acc: to_consume_acc + length as usize,
                        }
                    } else {
                        self.recv_raw().map_err(|_| ())?;
                    }
                }
            }
        }
    }

    /// Send bytes to the server.
    pub fn send_raw(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(bytes)?;

        Ok(())
    }

    /// Receive bytes from the server.
    pub fn recv_raw(&mut self) -> std::io::Result<usize> {
        let mut buffer = [0u8; 1024];
        let amt = self.stream.read(&mut buffer)?;

        let bytes = &buffer[..amt];

        self.buffer.extend_from_slice(bytes);

        Ok(amt)
    }
}
