//! Authentication-related types.

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    str::FromStr,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{impl_try_from, Atom},
    error::ValidationError,
    secret::Secret,
};

/// Authentication mechanism.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum AuthMechanism<'a> {
    /// The PLAIN SASL mechanism.
    ///
    /// ```imap
    /// AUTH=PLAIN
    /// ```
    ///
    /// ```text
    /// base64(b"<authorization identity>\x00<authentication identity>\x00<password>")
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

    /// OAuth 2.0 bearer token mechanism.
    ///
    /// ```imap
    /// AUTH=OAUTHBEARER
    /// ```
    ///
    /// ```text
    /// base64(b"n,a=<user>,\x01host=<host>\x01port=<port>\x01auth=Bearer <token>\x01\x01")
    /// ```
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/rfc7628>
    OAuthBearer,

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

    //
    // --- SHA-1 ---
    //
    /// SCRAM-SHA-1
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/rfc5802>
    ScramSha1,

    /// SCRAM-SHA-1-PLUS
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/rfc5802>
    ScramSha1Plus,

    //
    // --- SHA-2 ---
    //
    /// SCRAM-SHA-256
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/rfc7677>
    ScramSha256,

    /// SCRAM-SHA-256-PLUS
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/rfc7677>
    ScramSha256Plus,

    //
    // --- SHA-3 ---
    //
    /// SCRAM-SHA3-512
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/draft-melnikov-scram-sha3-512>
    ScramSha3_512,

    /// SCRAM-SHA3-512-PLUS
    ///
    /// # Reference(s):
    ///
    /// * <https://datatracker.ietf.org/doc/html/draft-melnikov-scram-sha3-512>
    ScramSha3_512Plus,

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
            "OAUTHBEARER" => Self::OAuthBearer,
            "XOAUTH2" => Self::XOAuth2,
            "SCRAM-SHA-1" => Self::ScramSha1,
            "SCRAM-SHA-1-PLUS" => Self::ScramSha1Plus,
            "SCRAM-SHA-256" => Self::ScramSha256,
            "SCRAM-SHA-256-PLUS" => Self::ScramSha256Plus,
            _ => Self::Other(AuthMechanismOther(atom)),
        }
    }
}

impl<'a> Display for AuthMechanism<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl<'a> AsRef<str> for AuthMechanism<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Plain => "PLAIN",
            Self::Login => "LOGIN",
            Self::OAuthBearer => "OAUTHBEARER",
            Self::XOAuth2 => "XOAUTH2",
            Self::ScramSha1 => "SCRAM-SHA-1",
            Self::ScramSha1Plus => "SCRAM-SHA-1-PLUS",
            Self::ScramSha256 => "SCRAM-SHA-256",
            Self::ScramSha256Plus => "SCRAM-SHA-256-PLUS",
            Self::ScramSha3_512 => "SCRAM-SHA3-512",
            Self::ScramSha3_512Plus => "SCRAM-SHA3-512-PLUS",
            Self::Other(other) => other.0.as_ref(),
        }
    }
}

impl FromStr for AuthMechanism<'static> {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AuthMechanism::try_from(s.to_string())
    }
}

/// An (unknown) authentication mechanism.
///
/// It's guaranteed that this type can't represent any mechanism from [`AuthMechanism`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct AuthMechanismOther<'a>(Atom<'a>);

/// Data line used, e.g., during AUTHENTICATE.
///
/// Holds the raw binary data, i.e., a `Vec<u8>`, *not* the BASE64 string.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum AuthenticateData<'a> {
    /// Continue SASL authentication.
    Continue(Secret<Cow<'a, [u8]>>),
    /// Cancel SASL authentication.
    ///
    /// "If the client wishes to cancel an authentication exchange,
    /// it issues a line consisting of a single "*"." (RFC 3501)
    Cancel,
}

impl<'a> AuthenticateData<'a> {
    pub fn r#continue<D>(data: D) -> Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        Self::Continue(Secret::new(data.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        assert!(AuthMechanism::try_from("plain").is_ok());
        assert!(AuthMechanism::try_from("login").is_ok());
        assert!(AuthMechanism::try_from("oauthbearer").is_ok());
        assert!(AuthMechanism::try_from("xoauth2").is_ok());
        assert!(AuthMechanism::try_from("xxxplain").is_ok());
        assert!(AuthMechanism::try_from("xxxlogin").is_ok());
        assert!(AuthMechanism::try_from("xxxxoauth2").is_ok());
    }
}
