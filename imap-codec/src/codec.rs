//! # (De)serialization of messages.
//!
//! You can `use imap_codec::Encoder` and call the `Encoder::encode(...)` to serialize a message.
//! Note that IMAP traces are not guaranteed to be UTF-8. Thus, be careful when using things like `std::str::from_utf8(...).unwrap()`.
//! It should generally be better not to think about IMAP as being UTF-8.
//! This is also why `Display` is not implemented.
//! All types implement `Debug`, though.
//!
//! ## Example
//!
//! ```
//! use imap_codec::{
//!     encode::Encoder,
//!     imap_types::command::{Command, CommandBody},
//!     CommandCodec,
//! };
//!
//! // Create some command.
//! let cmd = Command::new("A123", CommandBody::login("alice", "password").unwrap()).unwrap();
//!
//! // Encode the `cmd` into `out`.
//! let out = CommandCodec::default().encode(&cmd).dump();
//!
//! // Print the command.
//! // (Note that IMAP traces are not guaranteed to be valid UTF-8.)
//! println!("{}", std::str::from_utf8(&out).unwrap());
//! ```

pub mod decode;
pub mod encode;

/// Codec for greetings.
///
/// # Decoding
///
/// ```rust
/// # use imap_codec::{
/// #     decode::Decoder,
/// #     imap_types::{
/// #         core::Text,
/// #         response::{Code, Greeting, GreetingKind},
/// #     },
/// #     GreetingCodec,
/// #  };
/// let (remaining, greeting) =
///     GreetingCodec::decode(b"* OK [ALERT] Hello, World!\r\n<remaining>").unwrap();
///
/// assert_eq!(
///     greeting,
///     Greeting {
///         kind: GreetingKind::Ok,
///         code: Some(Code::Alert),
///         text: Text::try_from("Hello, World!").unwrap(),
///     }
/// );
/// assert_eq!(remaining, &b"<remaining>"[..])
/// ```
///
/// # Encoding
///
/// ```rust
/// # use imap_codec::{
/// #     encode::Encoder,
/// #     imap_types::{
/// #         core::Text,
/// #         response::{Code, Greeting, GreetingKind},
/// #     },
/// #     GreetingCodec,
/// #  };
/// let greeting = Greeting {
///     kind: GreetingKind::Ok,
///     code: Some(Code::Alert),
///     text: Text::try_from("Hello, World!").unwrap(),
/// };
///
/// let bytes = GreetingCodec::default().encode(&greeting).dump();
///
/// assert_eq!(bytes, &b"* OK [ALERT] Hello, World!\r\n"[..]);
/// ```
#[derive(Debug, Default)]
// We use `#[non_exhaustive]` to prevent users from using struct literal syntax.
//
// This allows to add configuration options later. For example, the
// codec could transparently replace all literals with non-sync literals.
#[non_exhaustive]
pub struct GreetingCodec;

/// Codec for commands.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct CommandCodec;

/// Codec for authenticate data lines.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct AuthenticateDataCodec;

/// Codec for responses.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ResponseCodec;

/// Codec for idle dones.
#[cfg(feature = "ext_idle")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_idle")))]
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct IdleDoneCodec;

macro_rules! impl_codec_new {
    ($codec:ty) => {
        impl $codec {
            /// Create codec with default configuration.
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

impl_codec_new!(GreetingCodec);
impl_codec_new!(CommandCodec);
impl_codec_new!(AuthenticateDataCodec);
impl_codec_new!(ResponseCodec);
#[cfg(feature = "ext_idle")]
impl_codec_new!(IdleDoneCodec);

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    #[cfg(feature = "ext_literal")]
    use imap_types::core::LiteralMode;
    use imap_types::{
        auth::AuthenticateData,
        command::{Command, CommandBody},
        core::{IString, Literal, NString, NonEmptyVec, Tag},
        fetch::MessageDataItem,
        mailbox::Mailbox,
        response::{Data, Greeting, GreetingKind, Response},
        secret::Secret,
    };

    use super::*;
    use crate::{
        decode::{CommandDecodeError, Decoder, GreetingDecodeError, ResponseDecodeError},
        testing::{
            kat_inverse_authenticate_data, kat_inverse_command, kat_inverse_greeting,
            kat_inverse_response,
        },
    };

    #[test]
    fn test_kat_inverse_greeting() {
        kat_inverse_greeting(&[
            (
                b"* OK ...\r\n".as_ref(),
                b"".as_ref(),
                Greeting::new(GreetingKind::Ok, None, "...").unwrap(),
            ),
            (
                b"* ByE .\r\n???",
                b"???",
                Greeting::new(GreetingKind::Bye, None, ".").unwrap(),
            ),
            (
                b"* preaUth x\r\n?",
                b"?",
                Greeting::new(GreetingKind::PreAuth, None, "x").unwrap(),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_command() {
        kat_inverse_command(&[
            (
                b"a nOOP\r\n".as_ref(),
                b"".as_ref(),
                Command::new("a", CommandBody::Noop).unwrap(),
            ),
            (
                b"a NooP\r\n???",
                b"???",
                Command::new("a", CommandBody::Noop).unwrap(),
            ),
            (
                b"a SeLECT {5}\r\ninbox\r\n",
                b"",
                Command::new(
                    "a",
                    CommandBody::Select {
                        mailbox: Mailbox::Inbox,
                    },
                )
                .unwrap(),
            ),
            (
                b"a SElECT {5}\r\ninbox\r\nxxx",
                b"xxx",
                Command::new(
                    "a",
                    CommandBody::Select {
                        mailbox: Mailbox::Inbox,
                    },
                )
                .unwrap(),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_response() {
        kat_inverse_response(&[
            (
                b"* SEARCH 1\r\n".as_ref(),
                b"".as_ref(),
                Response::Data(Data::Search(vec![NonZeroU32::new(1).unwrap()])),
            ),
            (
                b"* SEARCH 1\r\n???",
                b"???",
                Response::Data(Data::Search(vec![NonZeroU32::new(1).unwrap()])),
            ),
            (
                b"* 1 FETCH (RFC822 {5}\r\nhello)\r\n",
                b"",
                Response::Data(Data::Fetch {
                    seq: NonZeroU32::new(1).unwrap(),
                    items: NonEmptyVec::from(MessageDataItem::Rfc822(NString(Some(
                        IString::Literal(Literal::try_from(b"hello".as_ref()).unwrap()),
                    )))),
                }),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_authenticate_data() {
        kat_inverse_authenticate_data(&[(
            b"VGVzdA==\r\n".as_ref(),
            b"".as_ref(),
            AuthenticateData(Secret::new(b"Test".to_vec())),
        )]);
    }

    #[test]
    fn test_greeting_incomplete_failed() {
        let tests = [
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
    fn test_command_incomplete_failed() {
        let tests = [
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
                    #[cfg(feature = "ext_literal")]
                    mode: LiteralMode::Sync,
                }),
            ),
            #[cfg(feature = "ext_literal")]
            (
                b"a select {5+}\r\n".as_ref(),
                Err(CommandDecodeError::LiteralFound {
                    tag: Tag::try_from("a").unwrap(),
                    length: 5,
                    mode: LiteralMode::NonSync,
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
    fn test_response_incomplete_failed() {
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
            // LiteralAck treated as Incomplete
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
