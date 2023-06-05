#[cfg(feature = "ext_idle")]
use crate::extensions::idle::{idle_done, IdleDone};
use crate::{
    auth::{authenticate_data, AuthenticateData},
    command::{command, Command},
    response::{continue_req, greeting, response, Continue, Greeting, Response},
};

pub trait Decode<'a>: Sized + 'a {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// More data is needed.
    Incomplete,

    /// More data is needed (and further action may be necessary).
    ///
    /// The decoder stopped at the beginning of literal data. When in the role of a server, sending
    /// a continuation request may be necessary to agree to the receival of the remaining data.
    LiteralFound,

    // Decoding failed.
    Failed,
}

// -------------------------------------------------------------------------------------------------

impl<'a> Decode<'a> for Greeting<'a> {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError> {
        match greeting(input) {
            Ok((rem, grt)) => Ok((rem, grt)),
            Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
            Err(nom::Err::Failure(error)) => match error.code {
                nom::error::ErrorKind::Fix => Err(DecodeError::LiteralFound),
                _ => Err(DecodeError::Failed),
            },
            Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
        }
    }
}

impl<'a> Decode<'a> for Command<'a> {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError> {
        match command(input) {
            Ok((rem, cmd)) => Ok((rem, cmd)),
            Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
            Err(nom::Err::Failure(error)) => match error.code {
                nom::error::ErrorKind::Fix => Err(DecodeError::LiteralFound),
                _ => Err(DecodeError::Failed),
            },
            Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
        }
    }
}

impl<'a> Decode<'a> for AuthenticateData {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError> {
        match authenticate_data(input) {
            Ok((rem, auth_data)) => Ok((rem, auth_data)),
            Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
            Err(nom::Err::Failure(_)) => Err(DecodeError::Failed),
            Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
        }
    }
}

#[cfg(feature = "ext_idle")]
impl<'a> Decode<'a> for IdleDone {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError> {
        match idle_done(input) {
            Ok((rem, done)) => Ok((rem, done)),
            Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
            Err(nom::Err::Failure(_)) => Err(DecodeError::Failed),
            Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
        }
    }
}

impl<'a> Decode<'a> for Response<'a> {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError> {
        match response(input) {
            Ok((rem, rsp)) => Ok((rem, rsp)),
            Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
            Err(nom::Err::Failure(error)) => match error.code {
                nom::error::ErrorKind::Fix => Err(DecodeError::Incomplete),
                _ => Err(DecodeError::Failed),
            },
            Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
        }
    }
}

impl<'a> Decode<'a> for Continue<'a> {
    fn decode(input: &'a [u8]) -> Result<(&'a [u8], Self), DecodeError> {
        match continue_req(input) {
            Ok((rem, continue_req)) => Ok((rem, continue_req)),
            Err(nom::Err::Incomplete(_)) => Err(DecodeError::Incomplete),
            Err(nom::Err::Failure(error)) => match error.code {
                nom::error::ErrorKind::Fix => Err(DecodeError::Incomplete),
                _ => Err(DecodeError::Failed),
            },
            Err(nom::Err::Error(_)) => Err(DecodeError::Failed),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    #[cfg(feature = "ext_idle")]
    use crate::extensions::idle::IdleDone;
    use crate::{
        command::{Command, CommandBody},
        core::{IString, Literal, NString, NonEmptyVec},
        fetch::FetchAttributeValue,
        mailbox::Mailbox,
        response::{Data, Greeting, GreetingKind, Response},
    };

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
            let got = Greeting::decode(test);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
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
            let got = IdleDone::decode(test);

            dbg!((std::str::from_utf8(test).unwrap(), &expected, &got));

            assert_eq!(expected, got);
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
                        attributes: NonEmptyVec::from(FetchAttributeValue::Rfc822(NString(Some(
                            IString::Literal(Literal::try_from(b"hello".as_ref()).unwrap()),
                        )))),
                    }),
                )),
            ),
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
}
