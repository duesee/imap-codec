use crate::{codec::Codec, types::core::Atom};
use serde::Deserialize;
use std::fmt;

pub mod command;
pub mod core;
pub mod data_items;
pub mod datetime;
pub mod flag;
pub mod mailbox;
pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Capability {
    Imap4Rev1,
    Auth(AuthMechanism),
    LoginDisabled,
    StartTls,
    // ---
    Idle,             // RFC 2177
    MailboxReferrals, // RFC 2193
    LoginReferrals,   // RFC 2221
    SaslIr,           // RFC 4959
    Enable,           // RFC 5161
    // --- Other ---
    // TODO: Is this a good idea?
    // FIXME: mark this enum as non-exhaustive at least?
    Other(Atom),
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use Capability::*;

        match self {
            Imap4Rev1 => write!(f, "IMAP4REV1"),
            Auth(mechanism) => match mechanism {
                AuthMechanism::Plain => write!(f, "AUTH=PLAIN"),
                AuthMechanism::Login => write!(f, "AUTH=LOGIN"),
                AuthMechanism::Other(mech) => write!(f, "AUTH={}", mech),
            },
            LoginDisabled => write!(f, "LOGINDISABLED"),
            StartTls => write!(f, "STARTTLS"),
            Idle => write!(f, "IDLE"),
            MailboxReferrals => write!(f, "MAILBOX-REFERRALS"),
            LoginReferrals => write!(f, "LOGIN-REFERRALS"),
            SaslIr => write!(f, "SASL-IR"),
            Enable => write!(f, "ENABLE"),
            Other(atom) => write!(f, "{}", atom),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum AuthMechanism {
    // RFC4616: The PLAIN Simple Authentication and Security Layer (SASL) Mechanism
    // AUTH=PLAIN
    Plain,
    // TODO: where does it come from?
    // * draft-murchison-sasl-login-00: The LOGIN SASL Mechanism (?)
    // AUTH=LOGIN
    Login,
    Other(Atom),
}

impl Codec for AuthMechanism {
    fn serialize(&self) -> Vec<u8> {
        match self {
            AuthMechanism::Plain => b"PLAIN".to_vec(),
            AuthMechanism::Login => b"LOGIN".to_vec(),
            AuthMechanism::Other(atom) => atom.serialize(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
}

impl Codec for Sequence {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Sequence::Single(seq_no) => seq_no.serialize(),
            Sequence::Range(from, to) => {
                [&from.serialize(), b":".as_ref(), &to.serialize()].concat()
            }
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqNo {
    Value(u32),
    Unlimited,
}

impl Codec for SeqNo {
    fn serialize(&self) -> Vec<u8> {
        match self {
            SeqNo::Value(number) => number.to_string().into_bytes(),
            SeqNo::Unlimited => b"*".to_vec(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StoreType {
    Replace,
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StoreResponse {
    Answer,
    Silent,
}

#[cfg(test)]
mod test {
    use super::{SeqNo, Sequence};
    use crate::codec::Codec;

    #[test]
    fn test_sequence_serialize() {
        let tests = [
            (b"1".as_ref(), Sequence::Single(SeqNo::Value(1))),
            (b"*".as_ref(), Sequence::Single(SeqNo::Unlimited)), // TODO: is this a valid sequence?
            (
                b"1:*".as_ref(),
                Sequence::Range(SeqNo::Value(1), SeqNo::Unlimited),
            ),
        ];

        for (expected, test) in tests.iter() {
            let got = test.serialize();
            assert_eq!(*expected, got);
        }
    }
}
