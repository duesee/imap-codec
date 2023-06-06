use std::borrow::Cow;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    core::{impl_try_from, Atom, AtomError},
    secret::Secret,
};

/// Note: Defined by \[SASL\]
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

impl_try_from!(Atom<'a>, 'a, &'a [u8], AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, Vec<u8>, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, &'a str, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, String, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, Cow<'a, str>, AuthMechanism<'a>);

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
    pub fn verify(atom: &Atom<'a>) -> Result<(), AuthMechanismOtherError> {
        if matches!(
            atom.as_ref().to_ascii_lowercase().as_ref(),
            "plain" | "login",
        ) {
            return Err(AuthMechanismOtherError::Reserved);
        }

        Ok(())
    }

    pub fn inner(&self) -> &Atom<'a> {
        &self.0
    }
}

macro_rules! impl_try_from {
    ($from:ty) => {
        impl<'a> TryFrom<$from> for AuthMechanismOther<'a> {
            type Error = AuthMechanismOtherError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let atom = Atom::try_from(value)?;

                Self::verify(&atom)?;

                Ok(Self(atom))
            }
        }
    };
}

impl_try_from!(&'a [u8]);
impl_try_from!(Vec<u8>);
impl_try_from!(&'a str);
impl_try_from!(String);

impl<'a> TryFrom<Atom<'a>> for AuthMechanismOther<'a> {
    type Error = AuthMechanismOtherError;

    fn try_from(atom: Atom<'a>) -> Result<Self, Self::Error> {
        Self::verify(&atom)?;

        Ok(Self(atom))
    }
}

impl<'a> AsRef<str> for AuthMechanismOther<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuthMechanismOtherError {
    #[error(transparent)]
    Atom(#[from] AtomError),
    #[error("Reserved: Please use one of the typed variants")]
    Reserved,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticateData(pub Secret<Vec<u8>>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_failing() {
        assert!(AuthMechanismOther::try_from("plain").is_err());
        assert!(AuthMechanismOther::try_from("login").is_err());
    }
}
