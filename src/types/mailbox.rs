use crate::codec::Codec;
use crate::types::core::{AString, String as IMAPString};
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum MailboxWithWildcards {
    V1(String),
    V2(IMAPString),
}

impl std::fmt::Display for MailboxWithWildcards {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            MailboxWithWildcards::V1(str) => write!(f, "{}", str),
            MailboxWithWildcards::V2(imap_str) => write!(f, "{}", imap_str),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
// FIXME: prevent `Mailbox::Other("Inbox")`?
pub enum Mailbox {
    Inbox,
    Other(AString),
}

impl Codec for Mailbox {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Mailbox::Inbox => b"INBOX".to_vec(),
            Mailbox::Other(a_str) => a_str.serialize(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), String>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl fmt::Display for Mailbox {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", String::from_utf8(self.serialize()).unwrap())
    }
}

impl FromStr for Mailbox {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "inbox" {
            Ok(Mailbox::Inbox)
        } else {
            Ok(Mailbox::Other(AString::String(IMAPString::Quoted(
                s.to_string(),
            ))))
        }
    }
}
