use crate::{
    codec::Codec,
    types::core::{AString, String as IMAPString},
};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum MailboxWithWildcards {
    V1(String),
    V2(IMAPString),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Mailbox {
    Inbox,
    // FIXME: prevent `Mailbox::Other("Inbox")`?
    Other(AString),
}

impl Codec for Mailbox {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Mailbox::Inbox => b"INBOX".to_vec(),
            Mailbox::Other(a_str) => a_str.serialize(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Mailbox>
    where
        Self: Sized,
    {
        unimplemented!()
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
