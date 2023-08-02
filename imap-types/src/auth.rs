//! Authentication-related types.

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

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

/// Authentication mechanism.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AuthMechanism<'a> {
    /// The PLAIN SASL mechanism.
    ///
    /// ```imap
    /// AUTH=PLAIN
    /// ```
    ///
    /// ```text
    /// base64(b"<authenticate-id>\x00<authorize-id>\x00<password>")
    /// ```
    ///
    /// # Reference(s):
    ///
    /// * RFC4616: The PLAIN Simple Authentication and Security Layer (SASL) Mechanism
    Plain,

    /// The (non-standardized and slow) LOGIN SASL mechanism.
    ///
    /// ```imap
    /// AUTH=LOGIN
    /// ```
    ///
    /// ```text
    /// base64(b"<username>")
    /// base64(b"<password>")
    /// ```
    ///
    /// # Reference(s):
    ///
    /// + draft-murchison-sasl-login-00: The LOGIN SASL Mechanism
    Login,

    /// Google's OAuth 2.0 mechanism.
    ///
    /// ```imap
    /// AUTH=XOAUTH2
    /// ```
    ///
    /// ```text
    /// base64(b"user=<user>\x01auth=Bearer <token>\x01\x01")
    /// ```
    ///
    /// # Reference(s):
    ///
    /// * <https://developers.google.com/gmail/imap/xoauth2-protocol>
    XOAuth2,

    /// Some other (unknown) mechanism.
    Other(AuthMechanismOther<'a>),
}

impl_try_from!(Atom<'a>, 'a, &'a [u8], AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, Vec<u8>, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, &'a str, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, String, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, Cow<'a, str>, AuthMechanism<'a>);

impl<'a> From<Atom<'a>> for AuthMechanism<'a> {
    fn from(atom: Atom<'a>) -> Self {
        match atom.as_ref().to_ascii_uppercase().as_str() {
            "PLAIN" => Self::Plain,
            "LOGIN" => Self::Login,
            "XOAUTH2" => Self::XOAuth2,
            _ => Self::Other(AuthMechanismOther(atom)),
        }
    }
}

impl<'a> Display for AuthMechanism<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Plain => "PLAIN",
            Self::Login => "LOGIN",
            Self::XOAuth2 => "XOAUTH2",
            Self::Other(other) => other.as_ref(),
        })
    }
}

/// An (unknown) authentication mechanism.
///
/// It's guaranteed that this type can't represent any mechanism from [`AuthMechanism`].
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthMechanismOther<'a>(Atom<'a>);

impl<'a> AuthMechanismOther<'a> {
    pub fn validate(atom: &Atom<'a>) -> Result<(), AuthMechanismOtherError> {
        if matches!(
            atom.as_ref().to_ascii_lowercase().as_ref(),
            "plain" | "login" | "xoauth2",
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

                Self::validate(&atom)?;

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
        Self::validate(&atom)?;

        Ok(Self(atom))
    }
}

impl<'a> AsRef<str> for AuthMechanismOther<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Error during creation of an unknown authentication mechanism.
#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuthMechanismOtherError {
    /// Value is not a valid atom.
    #[error(transparent)]
    Atom(#[from] AtomError),
    /// Value would collide with [`AuthMechanism`].
    #[error("Reserved: Please use one of the typed variants")]
    Reserved,
}

/// Data line used, e.g., during AUTHENTICATE.
///
/// Holds the raw binary data, i.e., a `Vec<u8>`, *not* the BASE64 string.
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
        assert!(AuthMechanismOther::try_from("xoauth2").is_err());
    }
}
