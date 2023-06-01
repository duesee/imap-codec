//! # Serialization of messages
//!
//! All messages implement the `Encode` trait.
//! You can `use imap_codec::Encode` and call the `.encode()` (or `.encode().dump()`) method to serialize a message.
//! Note that IMAP traces are not guaranteed to be UTF-8. Thus, be careful when using things like `std::str::from_utf8(...).unwrap()`.
//! It should generally be better not to think about IMAP as being UTF-8.
//! This is also why `Display` is not implemented.
//! All types implement `Debug`, though.
//!
//! ## Example
//!
//! ```
//! use imap_codec::{
//!     codec::Encode,
//!     command::{Command, CommandBody},
//! };
//!
//! // Create some command.
//! let cmd = Command::new("A123", CommandBody::login("alice", "password").unwrap()).unwrap();
//!
//! // Encode the `cmd` into `out`.
//! let out = cmd.encode().dump();
//!
//! // Print the command.
//! // (Note that IMAP traces are not guaranteed to be valid UTF-8.)
//! println!("{}", std::str::from_utf8(&out).unwrap());
//! ```

pub use decode::{Decode, DecodeError};
#[cfg(any(
    feature = "ext_compress",
    feature = "ext_enable",
    feature = "ext_idle",
    feature = "ext_literal",
    feature = "ext_quota",
))]
pub(crate) use encode::Encoder;
pub use encode::{Action, Encode, EncodeContext, Encoded};

mod decode;
mod encode;

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        command::{Command, CommandBody},
        core::{IString, Literal, NString, NonEmptyVec},
        fetch::FetchAttributeValue,
        mailbox::Mailbox,
        response::{Data, Greeting, GreetingKind, Response},
        testing::{kat_inverse_command, kat_inverse_greeting, kat_inverse_response},
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
                    seq_or_uid: NonZeroU32::new(1).unwrap(),
                    attributes: NonEmptyVec::from(FetchAttributeValue::Rfc822(NString(Some(
                        IString::Literal(Literal::try_from(b"hello".as_ref()).unwrap()),
                    )))),
                }),
            ),
        ]);
    }

    #[test]
    fn test_greeting_incomplete_failed() {
        let tests = [
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
            let got = Greeting::decode(test);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_command_incomplete_failed() {
        let tests = [
            // Incomplete
            (b"a".as_ref(), Err(DecodeError::Incomplete)),
            (b"a ".as_ref(), Err(DecodeError::Incomplete)),
            (b"a n".as_ref(), Err(DecodeError::Incomplete)),
            (b"a no".as_ref(), Err(DecodeError::Incomplete)),
            (b"a noo".as_ref(), Err(DecodeError::Incomplete)),
            (b"a noop".as_ref(), Err(DecodeError::Incomplete)),
            (b"a noop\r".as_ref(), Err(DecodeError::Incomplete)),
            // LiteralAckRequired
            (b"a select {5}\r\n".as_ref(), Err(DecodeError::LiteralFound)),
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
            let got = Command::decode(test);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_response_incomplete_failed() {
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
            // LiteralAck treated as Incomplete
            (
                b"* 1 FETCH (RFC822 {5}\r\n".as_ref(),
                Err(DecodeError::Incomplete),
            ),
            // Failed
            (b"*  search 1 2 3\r\n".as_ref(), Err(DecodeError::Failed)),
            (b"A search\r\n".as_ref(), Err(DecodeError::Failed)),
        ];

        for (test, expected) in tests {
            let got = Response::decode(test);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
        }
    }

    // #[test]
    // fn test_decode_authenticate_data() {
    //     let tests = [
    //         // Ok
    //         // Incomplete
    //         // Failed
    //     ];
    //
    //     for (test, expected) in tests {
    //         let got = AuthenticateData::decode(test);
    //
    //         dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));
    //
    //         assert_eq!(expected, got);
    //     }
    // }
}
