use crate::types::core::Atom;
use serde::Deserialize;
use std::fmt;

pub mod command;
pub mod core;
pub mod data_items;
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
    Enable,           // RFC 5161
    MailboxReferrals, // RFC 2193
    LoginReferrals,   // RFC 2221
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
            Enable => write!(f, "ENABLE"),
            MailboxReferrals => write!(f, "MAILBOX-REFERRALS"),
            LoginReferrals => write!(f, "LOGIN-REFERRALS"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqNo {
    Value(u32),
    Unlimited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sequence {
    Single(SeqNo),
    Range(SeqNo, SeqNo),
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
