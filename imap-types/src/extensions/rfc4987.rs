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

use std::{borrow::Cow, convert::TryFrom, io::Write};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    codec::{Context, Encode},
    core::Atom,
    rfc3501::core::impl_try_from_try_from,
};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompressionAlgorithm {
    Deflate,
}

impl_try_from_try_from!(Atom, 'a, &'a str, CompressionAlgorithm);
impl_try_from_try_from!(Atom, 'a, &'a [u8], CompressionAlgorithm);
impl_try_from_try_from!(Atom, 'a, Vec<u8>, CompressionAlgorithm);
impl_try_from_try_from!(Atom, 'a, String, CompressionAlgorithm);
impl_try_from_try_from!(Atom, 'a, Cow<'a, str>, CompressionAlgorithm);

impl<'a> TryFrom<Atom<'a>> for CompressionAlgorithm {
    type Error = ();

    fn try_from(atom: Atom<'a>) -> Result<Self, ()> {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "deflate" => Ok(Self::Deflate),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for CompressionAlgorithm {
    fn as_ref(&self) -> &str {
        match self {
            CompressionAlgorithm::Deflate => "deflate",
        }
    }
}

impl Encode for CompressionAlgorithm {
    fn encode(&self, writer: &mut impl Write, _: &Context) -> std::io::Result<()> {
        match self {
            CompressionAlgorithm::Deflate => writer.write_all(b"DEFLATE"),
        }
    }
}
