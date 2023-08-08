//! The IMAP ENABLE Extension
//!
//! This extension extends ...
//!
//! * the [Capability](crate::response::Capability) enum with a new variant [Capability::Enable](crate::response::Capability#variant.Enable),
//! * the [CommandBody](crate::command::CommandBody) enum with a new variant [CommandBody::Enable](crate::command::CommandBody#variant.Enable), and
//! * the [Data](crate::response::Data) enum with a new variant [Data::Enabled](crate::response::Data#variant.Enabled).

use std::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    command::CommandBody,
    core::{Atom, AtomError, NonEmptyVec},
};

impl<'a> CommandBody<'a> {
    pub fn enable<C>(capabilities: C) -> Result<Self, C::Error>
    where
        C: TryInto<NonEmptyVec<CapabilityEnable<'a>>>,
    {
        Ok(CommandBody::Enable {
            capabilities: capabilities.try_into()?,
        })
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CapabilityEnable<'a> {
    Utf8(Utf8Kind),
    #[cfg(feature = "ext_condstore_qresync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_condstore_qresync")))]
    CondStore,
    Other(CapabilityEnableOther<'a>),
}

impl<'a> TryFrom<&'a str> for CapabilityEnable<'a> {
    type Error = AtomError;

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
            Self::Other(other) => write!(f, "{}", other.0),
        }
    }
}

/// An (unknown) capability.
///
/// It's guaranteed that this type can't represent any capability from [`CapabilityEnable`].
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityEnableOther<'a>(Atom<'a>);

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            CapabilityEnable::Other(CapabilityEnableOther(Atom::unvalidated("utf")))
        );
        assert_eq!(
            CapabilityEnable::try_from("xxxxx").unwrap(),
            CapabilityEnable::Other(CapabilityEnableOther(Atom::unvalidated("xxxxx")))
        );
    }
}
