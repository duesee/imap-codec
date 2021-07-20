use std::io::Write;

#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::{codec::Encode, types::core::Atom};

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
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl Encode for AuthMechanism {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            AuthMechanism::Plain => writer.write_all(b"PLAIN"),
            AuthMechanism::Login => writer.write_all(b"LOGIN"),
            AuthMechanism::Other(atom) => atom.encode(writer),
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompressionAlgorithm {
    Deflate,
}

impl Encode for CompressionAlgorithm {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            CompressionAlgorithm::Deflate => writer.write_all(b"DEFLATE"),
        }
    }
}
