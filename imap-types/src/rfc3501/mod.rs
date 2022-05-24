use std::{
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::core::Atom;

pub mod address;
pub mod body;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
pub mod fetch_attributes;
pub mod flag;
pub mod mailbox;
pub mod response;
pub mod section;
pub mod sequence;
pub mod status_attributes;

/// Note: Defined by [SASL]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuthMechanism<'a> {
    // RFC4616: The PLAIN Simple Authentication and Security Layer (SASL) Mechanism
    // AUTH=PLAIN
    Plain,
    // TODO: where does it come from?
    // * draft-murchison-sasl-login-00: The LOGIN SASL Mechanism (?)
    // AUTH=LOGIN
    Login,
    Other(AuthMechanismOther<'a>),
}

impl<'a> From<Atom<'a>> for AuthMechanism<'a> {
    fn from(value: Atom<'a>) -> Self {
        match value.to_lowercase().as_str() {
            "plain" => AuthMechanism::Plain,
            "login" => AuthMechanism::Login,
            _ => AuthMechanism::Other(AuthMechanismOther(value)),
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthMechanismOther<'a>(pub(crate) Atom<'a>);

impl<'a> TryFrom<&'a str> for AuthMechanismOther<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl<'a> TryFrom<String> for AuthMechanismOther<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Atom::try_from(value)?.try_into()
    }
}

impl<'a> TryFrom<Atom<'a>> for AuthMechanismOther<'a> {
    type Error = ();

    fn try_from(value: Atom<'a>) -> Result<Self, ()> {
        match value.to_lowercase().as_str() {
            "plain" | "login" => Err(()),
            _ => Ok(AuthMechanismOther(value)),
        }
    }
}

impl<'a> Display for AuthMechanismOther<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}