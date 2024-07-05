//! The IMAP ENABLE Extension
//!
//! This extension extends ...
//!
//! * the [Capability](crate::response::Capability) enum with a new variant [Capability::Enable](crate::response::Capability#variant.Enable),
//! * the [CommandBody] enum with a new variant [CommandBody::Enable], and
//! * the [Data](crate::response::Data) enum with a new variant [Data::Enabled](crate::response::Data#variant.Enabled).

use std::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    command::CommandBody,
    core::{Atom, Vec1},
    error::ValidationError,
};

impl<'a> CommandBody<'a> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the ENABLE capability.
    /// </div>
    pub fn enable<C>(capabilities: C) -> Result<Self, C::Error>
    where
        C: TryInto<Vec1<CapabilityEnable<'a>>>,
    {
        Ok(CommandBody::Enable {
            capabilities: capabilities.try_into()?,
        })
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum CapabilityEnable<'a> {
    Utf8(Utf8Kind),
    #[cfg(feature = "ext_condstore_qresync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_condstore_qresync")))]
    CondStore,
    #[cfg(feature = "ext_metadata")]
    /// Client can handle unsolicited server annotations and mailbox annotations.
    Metadata,
    #[cfg(feature = "ext_metadata")]
    /// Client can handle server annotations.
    MetadataServer,
    Other(CapabilityEnableOther<'a>),
}

impl<'a> TryFrom<&'a str> for CapabilityEnable<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self::from(Atom::try_from(value)?))
    }
}

impl<'a> From<Atom<'a>> for CapabilityEnable<'a> {
    fn from(atom: Atom<'a>) -> Self {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "utf8=accept" => Self::Utf8(Utf8Kind::Accept),
            "utf8=only" => Self::Utf8(Utf8Kind::Only),
            #[cfg(feature = "ext_condstore_qresync")]
            "condstore" => Self::CondStore,
            #[cfg(feature = "ext_metadata")]
            "metadata" => Self::Metadata,
            #[cfg(feature = "ext_metadata")]
            "metadata-server" => Self::MetadataServer,
            _ => Self::Other(CapabilityEnableOther(atom)),
        }
    }
}

impl<'a> Display for CapabilityEnable<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Utf8(kind) => write!(f, "UTF8={}", kind),
            #[cfg(feature = "ext_condstore_qresync")]
            Self::CondStore => write!(f, "CONDSTORE"),
            #[cfg(feature = "ext_metadata")]
            Self::Metadata => write!(f, "METADATA"),
            #[cfg(feature = "ext_metadata")]
            Self::MetadataServer => write!(f, "METADATA-SERVER"),
            Self::Other(other) => write!(f, "{}", other.0),
        }
    }
}

/// An (unknown) capability.
///
/// It's guaranteed that this type can't represent any capability from [`CapabilityEnable`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct CapabilityEnableOther<'a>(Atom<'a>);

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum Utf8Kind {
    Accept,
    Only,
}

impl Display for Utf8Kind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Accept => "ACCEPT",
            Self::Only => "ONLY",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_capability_enable() {
        assert_eq!(
            CapabilityEnable::from(Atom::try_from("utf8=only").unwrap()),
            CapabilityEnable::Utf8(Utf8Kind::Only)
        );
        assert_eq!(
            CapabilityEnable::from(Atom::try_from("utf8=accept").unwrap()),
            CapabilityEnable::Utf8(Utf8Kind::Accept)
        );
        assert_eq!(
            CapabilityEnable::try_from("utf").unwrap(),
            CapabilityEnable::Other(CapabilityEnableOther(Atom::try_from("utf").unwrap()))
        );
        assert_eq!(
            CapabilityEnable::try_from("xxxxx").unwrap(),
            CapabilityEnable::Other(CapabilityEnableOther(Atom::try_from("xxxxx").unwrap()))
        );
    }
}
