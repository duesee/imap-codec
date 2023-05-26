use std::io::Write;

pub use imap_types::extensions::literal::*;

use crate::codec::{CoreEncode, EncodeContext};

impl CoreEncode for LiteralCapability {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Plus => writer.write_all(b"LITERAL+"),
            Self::Minus => writer.write_all(b"LITERAL-"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command::{Command, CommandBody},
        core::{Literal, NonEmptyVec},
        response::{data::Capability, Code, Greeting},
        testing::{kat_inverse_command, kat_inverse_greeting},
    };

    #[test]
    fn test_kat_inverse_command_login_literal_plus() {
        kat_inverse_command(&[
            (
                b"A LOGIN {0}\r\n {1}\r\nA\r\n".as_ref(),
                b"".as_ref(),
                Command::new(
                    "A",
                    CommandBody::login(
                        Literal::try_from("").unwrap(),
                        Literal::try_from("A").unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
            (
                b"A LOGIN {1}\r\nA {2}\r\nAB\r\n?".as_ref(),
                b"?".as_ref(),
                Command::new(
                    "A",
                    CommandBody::login(
                        Literal::try_from("A").unwrap(),
                        Literal::try_from("AB").unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
            (
                b"A LOGIN {0+}\r\n {1+}\r\nA\r\n??".as_ref(),
                b"??".as_ref(),
                Command::new(
                    "A",
                    CommandBody::login(
                        Literal::try_from("").unwrap().into_non_sync(),
                        Literal::try_from("A").unwrap().into_non_sync(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
            (
                b"A LOGIN {1+}\r\nA {2+}\r\nAB\r\n???".as_ref(),
                b"???".as_ref(),
                Command::new(
                    "A",
                    CommandBody::login(
                        Literal::try_from("A").unwrap().into_non_sync(),
                        Literal::try_from("AB").unwrap().into_non_sync(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_greeting_capability_literal_plus() {
        kat_inverse_greeting(&[
            (
                b"* OK [CAPABILITY LITERAL+] ...\r\n".as_ref(),
                b"".as_ref(),
                Greeting::ok(
                    Some(Code::Capability(NonEmptyVec::from(Capability::Literal(
                        LiteralCapability::Plus,
                    )))),
                    "...",
                )
                .unwrap(),
            ),
            (
                b"* OK [CAPABILITY LITERAL-] ...\r\n?",
                b"?",
                Greeting::ok(
                    Some(Code::Capability(NonEmptyVec::from(Capability::Literal(
                        LiteralCapability::Minus,
                    )))),
                    "...",
                )
                .unwrap(),
            ),
        ]);
    }
}
