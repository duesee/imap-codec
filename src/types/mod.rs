use crate::{codec::Serialize, types::core::Atom};
use serde::Deserialize;
use std::io::Write;

pub mod address;
pub mod body;
pub mod command;
pub mod core;
pub mod data_items;
pub mod datetime;
pub mod envelope;
pub mod flag;
pub mod mailbox;
pub mod response;
pub mod sequence;

/// Note: Defined by [SASL]
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

impl Serialize for AuthMechanism {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            AuthMechanism::Plain => writer.write_all(b"PLAIN"),
            AuthMechanism::Login => writer.write_all(b"LOGIN"),
            AuthMechanism::Other(atom) => atom.serialize(writer),
        }
    }
}
