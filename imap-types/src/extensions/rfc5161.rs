//! The IMAP ENABLE Extension
//!
//! This extension extends ...
//!
//! * the [Capability](crate::response::Capability) enum with a new variant [Capability::Enable](crate::response::Capability#variant.Enable),
//! * the [CommandBody](crate::command::CommandBody) enum with a new variant [CommandBody::Enable](crate::command::CommandBody#variant.Enable), and
//! * the [Data](crate::response::Data) enum with a new variant [Data::Enabled](crate::response::Data#variant.Enabled).

use std::io::Write;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    codec::{Context, Encode},
    core::Atom,
};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityEnable<'a> {
    Utf8(Utf8Kind),
    Other(Atom<'a>),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Utf8Kind {
    Accept,
    Only,
}

impl<'a> Encode for CapabilityEnable<'a> {
    fn encode(&self, writer: &mut impl Write, ctx: &Context) -> std::io::Result<()> {
        match self {
            Self::Utf8(Utf8Kind::Accept) => writer.write_all(b"UTF8=ACCEPT"),
            Self::Utf8(Utf8Kind::Only) => writer.write_all(b"UTF8=ONLY"),
            Self::Other(atom) => atom.encode(writer, ctx),
        }
    }
}
