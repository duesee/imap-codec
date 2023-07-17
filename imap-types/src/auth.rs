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

use crate::{
    core::{impl_try_from, Atom},
    secret::Secret,
};

/// Authentication mechanism.
///
/// It's recommended to use the pre-defined constants, such as, [`AuthMechanism::PLAIN`]. Still, you
/// can also (try to) construct an authentication mechanism from a value.
///
/// ```rust
/// use imap_types::{auth::AuthMechanism, core::Atom};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(AuthMechanism::PLAIN, AuthMechanism::try_from("plain")?);
/// assert_eq!(
///     AuthMechanism::PLAIN,
///     AuthMechanism::try_from(b"PLAIN".as_ref())?,
/// );
/// assert_eq!(
///     AuthMechanism::PLAIN,
///     AuthMechanism::from(Atom::try_from("pLAiN")?)
/// );
///
/// let mechanism = AuthMechanism::try_from(b"login".as_ref())?;
///
/// match mechanism {
///     AuthMechanism::PLAIN => {}
///     AuthMechanism::LOGIN => {}
///     _ => {}
/// }
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthMechanism<'a>(Inner<'a>);

impl<'a> AuthMechanism<'a> {
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
    pub const PLAIN: AuthMechanism<'static> = AuthMechanism(Inner::Plain);

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
    pub const LOGIN: AuthMechanism<'static> = AuthMechanism(Inner::Login);

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
    pub const XOAUTH2: AuthMechanism<'static> = AuthMechanism(Inner::XOAuth2);
}

impl_try_from!(Atom<'a>, 'a, &'a [u8], AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, Vec<u8>, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, &'a str, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, String, AuthMechanism<'a>);
impl_try_from!(Atom<'a>, 'a, Cow<'a, str>, AuthMechanism<'a>);

impl<'a> From<Atom<'a>> for AuthMechanism<'a> {
    fn from(atom: Atom<'a>) -> Self {
        match atom.as_ref().to_ascii_uppercase().as_str() {
            "PLAIN" => Self::PLAIN,
            "LOGIN" => Self::LOGIN,
            "XOAUTH2" => Self::XOAUTH2,
            _ => Self(Inner::Other(atom)),
        }
    }
}

impl<'a> Display for AuthMechanism<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self.0 {
            Inner::Plain => "PLAIN",
            Inner::Login => "LOGIN",
            Inner::XOAuth2 => "XOAUTH2",
            Inner::Other(other) => other.as_ref(),
        })
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Inner<'a> {
    Plain,
    Login,
    XOAuth2,
    Other(Atom<'a>),
}

/// Data line used, e.g., during AUTHENTICATE.
///
/// Holds the raw binary data, i.e., a `Vec<u8>`, *not* the BASE64 string.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticateData(pub Secret<Vec<u8>>);
