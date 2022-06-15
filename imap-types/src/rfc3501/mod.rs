use std::convert::{TryFrom, TryInto};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
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
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl<'a> AuthMechanism<'a> {
    pub fn other<O>(other: O) -> Result<Self, O::Error>
    where
        O: TryInto<AuthMechanismOther<'a>>,
    {
        Ok(AuthMechanism::Other(other.try_into()?))
    }
}

impl<'a> TryFrom<&'a str> for AuthMechanism<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, ()> {
        match value.to_ascii_lowercase().as_str() {
            "plain" => Ok(AuthMechanism::Plain),
            "login" => Ok(AuthMechanism::Login),
            _ => {
                let inner = Atom::try_from(value)?;
                Ok(AuthMechanism::Other(AuthMechanismOther { inner }))
            }
        }
    }
}

impl<'a> TryFrom<String> for AuthMechanism<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        match value.to_ascii_lowercase().as_str() {
            "plain" => Ok(AuthMechanism::Plain),
            "login" => Ok(AuthMechanism::Login),
            _ => {
                let inner = Atom::try_from(value)?;
                Ok(AuthMechanism::Other(AuthMechanismOther { inner }))
            }
        }
    }
}

impl<'a> From<Atom<'a>> for AuthMechanism<'a> {
    fn from(inner: Atom<'a>) -> Self {
        match inner.to_ascii_lowercase().as_str() {
            "plain" => AuthMechanism::Plain,
            "login" => AuthMechanism::Login,
            _ => AuthMechanism::Other(AuthMechanismOther { inner }),
        }
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthMechanismOther<'a> {
    pub(crate) inner: Atom<'a>,
}

impl<'a> AuthMechanismOther<'a> {
    pub fn inner(&self) -> &Atom<'a> {
        &self.inner
    }
}

impl<'a> TryFrom<Atom<'a>> for AuthMechanismOther<'a> {
    type Error = ();

    fn try_from(inner: Atom<'a>) -> Result<Self, ()> {
        match inner.to_ascii_lowercase().as_str() {
            "plain" | "login" => Err(()),
            _ => Ok(AuthMechanismOther { inner }),
        }
    }
}

impl<'a> TryFrom<&'a str> for AuthMechanismOther<'a> {
    type Error = ();

    fn try_from(inner: &'a str) -> Result<Self, ()> {
        match inner.to_ascii_lowercase().as_str() {
            "plain" | "login" => Err(()),
            _ => Ok(AuthMechanismOther {
                inner: Atom::try_from(inner)?,
            }),
        }
    }
}
