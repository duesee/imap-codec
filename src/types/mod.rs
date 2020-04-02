use serde::Deserialize;
use std::fmt;

pub mod command;
pub mod core;
pub mod data_items;
pub mod mailbox;
pub mod message_attributes;
pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Capability {
    Imap4Rev1,
    Auth(AuthMechanism),
    LoginDisabled,
    StartTls,
    // ---
    Idle,           // RFC 2177
    Enable,         // RFC 5161
    LoginReferrals, // RFC 2221
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use Capability::*;

        match self {
            Imap4Rev1 => write!(f, "IMAP4REV1"),
            Auth(mechanism) => match mechanism {
                AuthMechanism::Plain => write!(f, "AUTH=PLAIN"),
                AuthMechanism::Other(mech) => write!(f, "AUTH={}", mech),
            },
            LoginDisabled => write!(f, "LOGINDISABLED"),
            StartTls => write!(f, "STARTTLS"),
            Idle => write!(f, "IDLE"),
            Enable => write!(f, "ENABLE"),
            LoginReferrals => write!(f, "LOGIN-REFERRALS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum AuthMechanism {
    Plain,
    Other(String),
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
