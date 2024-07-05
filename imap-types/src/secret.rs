//! Handling of secret values.
//!
//! This module provides a `Secret<T>` ensuring that sensitive values are not
//! `Debug`-printed by accident.

use std::fmt::{Debug, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A wrapper to ensure that secrets are redacted during `Debug`-printing.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Eq, Hash, PartialEq, ToStatic)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Crate a new secret.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Expose the inner secret.
    pub fn declassify(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Debug for Secret<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(debug_assertions))]
        return write!(f, "/* REDACTED */");
        #[cfg(debug_assertions)]
        return self.0.fmt(f);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        command::{Command, CommandBody},
        core::{AString, Atom, Literal, Quoted},
    };

    #[test]
    #[cfg(not(debug_assertions))]
    #[allow(clippy::redundant_clone)]
    fn test_that_secret_is_redacted() {
        use super::Secret;
        use crate::auth::{AuthMechanism, AuthenticateData};

        let secret = Secret("xyz123");
        let got = format!("{:?}", secret);
        println!("{}", got);
        assert!(!got.contains("xyz123"));

        println!("-----");

        let tests = vec![
            CommandBody::login("alice", "xyz123")
                .unwrap()
                .tag("A")
                .unwrap(),
            CommandBody::authenticate_with_ir(AuthMechanism::Plain, b"xyz123".as_ref())
                .tag("A")
                .unwrap(),
        ];

        for test in tests.into_iter() {
            let got = format!("{:?}", test);
            println!("Debug: {}", got);
            assert!(got.contains("/* REDACTED */"));
            assert!(!got.contains("xyz123"));
            assert!(!got.contains("eHl6MTIz"));

            println!();
        }

        println!("-----");

        let tests = [
            AuthenticateData::r#continue(b"xyz123".to_vec()),
            AuthenticateData::r#continue(b"xyz123".to_vec()),
        ];

        for test in tests {
            let got = format!("{:?}", test);
            println!("Debug: {}", got);
            assert!(got.contains("/* REDACTED */"));
            assert!(!got.contains("xyz123"));
            assert!(!got.contains("eHl6MTIz"));
        }
    }

    #[test]
    fn test_that_secret_has_no_side_effects_on_eq() {
        assert_ne!(
            Command::new(
                "A",
                CommandBody::login(
                    AString::from(Atom::try_from("user").unwrap()),
                    AString::from(Atom::try_from("pass").unwrap()),
                )
                .unwrap(),
            ),
            Command::new(
                "A",
                CommandBody::login(
                    AString::from(Atom::try_from("user").unwrap()),
                    AString::from(Quoted::try_from("pass").unwrap()),
                )
                .unwrap(),
            )
        );

        assert_ne!(
            Command::new(
                "A",
                CommandBody::login(
                    Literal::try_from("").unwrap(),
                    Literal::try_from("A").unwrap(),
                )
                .unwrap(),
            ),
            Command::new(
                "A",
                CommandBody::login(
                    Literal::try_from("").unwrap(),
                    Literal::try_from("A").unwrap().into_non_sync(),
                )
                .unwrap(),
            )
        );
    }
}
