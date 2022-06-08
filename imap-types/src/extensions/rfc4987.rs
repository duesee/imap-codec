//! The IMAP COMPRESS Extension
//!
//! This extension defines a new type ...
//!
//! * [CompressionAlgorithm](crate::extensions::rfc4987::CompressionAlgorithm)
//!
//! ... and extends ...
//!
//! * the [Capability](crate::response::Capability) enum with a new variant [Capability::Compress](crate::response::Capability#variant.Compress),
//! * the [Command](crate::command::Command) enum with a new variant [Command::Compress](crate::command::Command#variant.Compress), and
//! * the [Code](crate::response::Code) enum with a new variant [Code::CompressionActive](crate::response::Code#variant.CompressionActive).

use std::io::Write;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Encode;

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
