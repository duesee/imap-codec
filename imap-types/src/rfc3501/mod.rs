use std::{borrow::Cow, convert::TryFrom};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::rfc3501::core::{impl_try_from, impl_try_from_try_from, Atom};

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

impl_try_from!(Atom, 'a, &'a [u8], AuthMechanism<'a>);
impl_try_from!(Atom, 'a, Vec<u8>, AuthMechanism<'a>);
impl_try_from!(Atom, 'a, &'a str, AuthMechanism<'a>);
impl_try_from!(Atom, 'a, String, AuthMechanism<'a>);
impl_try_from!(Atom, 'a, Cow<'a, str>, AuthMechanism<'a>);

impl<'a> From<Atom<'a>> for AuthMechanism<'a> {
    fn from(inner: Atom<'a>) -> Self {
        match inner.as_ref().to_ascii_lowercase().as_str() {
            "plain" => AuthMechanism::Plain,
            "login" => AuthMechanism::Login,
            _ => AuthMechanism::Other(AuthMechanismOther(inner)),
        }
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthMechanismOther<'a>(Atom<'a>);

impl<'a> AuthMechanismOther<'a> {
    pub fn inner(&self) -> &Atom<'a> {
        &self.0
    }
}

impl_try_from_try_from!(Atom, 'a, &'a [u8], AuthMechanismOther<'a>);
impl_try_from_try_from!(Atom, 'a, Vec<u8>, AuthMechanismOther<'a>);
impl_try_from_try_from!(Atom, 'a, &'a str, AuthMechanismOther<'a>);
impl_try_from_try_from!(Atom, 'a, String, AuthMechanismOther<'a>);

impl<'a> TryFrom<Atom<'a>> for AuthMechanismOther<'a> {
    type Error = ();

    fn try_from(atom: Atom<'a>) -> Result<Self, ()> {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "plain" | "login" => Err(()),
            _ => Ok(Self(atom)),
        }
    }
}

impl<'a> AsRef<str> for AuthMechanismOther<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
