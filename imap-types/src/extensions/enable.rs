//! The IMAP ENABLE Extension
//!
//! This extension extends ...
//!
//! * the [Capability](crate::response::Capability) enum with a new variant [Capability::Enable](crate::response::Capability#variant.Enable),
//! * the [CommandBody](crate::command::CommandBody) enum with a new variant [CommandBody::Enable](crate::command::CommandBody#variant.Enable), and
//! * the [Data](crate::response::Data) enum with a new variant [Data::Enabled](crate::response::Data#variant.Enabled).

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    command::CommandBody,
    core::{Atom, NonEmptyVec},
    response::Data,
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

impl<'a> Data<'a> {
    // TODO
    // pub fn enable() -> Self {
    //     unimplemented!()
    // }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityEnable<'a> {
    Utf8(Utf8Kind),
    #[cfg(feature = "ext_condstore_qresync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_condstore_qresync")))]
    CondStore,
    Other(CapabilityEnableOther<'a>),
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

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityEnableOther<'a>(Atom<'a>);

impl<'a> CapabilityEnableOther<'a> {
    pub fn inner(&self) -> &Atom<'a> {
        &self.0
    }
}

impl<'a> TryFrom<Atom<'a>> for CapabilityEnableOther<'a> {
    type Error = CapabilityEnableOtherError;

    fn try_from(value: Atom<'a>) -> Result<Self, Self::Error> {
        match value.as_ref().to_ascii_lowercase().as_ref() {
            "utf8=accept" | "utf8=only" => Err(CapabilityEnableOtherError::Reserved),
            _ => Ok(Self(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum CapabilityEnableOtherError {
    #[error("Reserved: Please use one of the typed variants")]
    Reserved,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Utf8Kind {
    Accept,
    Only,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_capability_enable_other() {
        assert_eq!(
            CapabilityEnable::from(Atom::try_from("utf8=only").unwrap()),
            CapabilityEnable::Utf8(Utf8Kind::Only)
        );
        assert_eq!(
            CapabilityEnable::from(Atom::try_from("utf8=accept").unwrap()),
            CapabilityEnable::Utf8(Utf8Kind::Accept)
        );
        assert_eq!(
            CapabilityEnableOther::try_from(Atom::try_from("utf8=only").unwrap()),
            Err(CapabilityEnableOtherError::Reserved)
        );
        assert_eq!(
            CapabilityEnableOther::try_from(Atom::try_from("utf8=accept").unwrap()),
            Err(CapabilityEnableOtherError::Reserved)
        );
    }
}
