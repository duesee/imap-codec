use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::core::{AString, IString, Literal};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
// Note: The implementation of these traits does agree:
//       `PartialEq` is just a thin wrapper that ensures constant-time comparison.
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Hash)]
pub struct Secret<T>(T);

/// A trait to ensure that secrets are neither logged nor compared in non-constant time.
impl<T> Secret<T> {
    /// Crate a new secret.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Expose the inner secret (opting-out of all guarantees).
    pub fn declassify(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Secret<T>
where
    T: AsRef<[u8]>,
{
    /// Compare this secret value with another value in constant time.
    ///
    /// Note: The comparison is made by converting both values as bytes first.
    pub fn compare_with<B>(&self, other: B) -> bool
    where
        B: AsRef<[u8]>,
    {
        self.declassify().as_ref().ct_eq(other.as_ref()).unwrap_u8() == 1
    }
}

impl<T> PartialEq for Secret<T>
where
    T: CompareCT<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.declassify().compare_ct(&other.0)
    }
}

impl<T> Eq for Secret<T> where T: CompareCT<T> {}

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

pub trait CompareCT<T> {
    #[must_use]
    fn compare_ct(&self, other: &T) -> bool;
}

impl<'a, T> CompareCT<T> for Cow<'a, [u8]>
where
    T: AsRef<[u8]>,
{
    fn compare_ct(&self, other: &T) -> bool {
        self.as_ref().ct_eq(other.as_ref()).unwrap_u8() == 1
    }
}

impl<T> CompareCT<T> for Vec<u8>
where
    T: AsRef<[u8]>,
{
    fn compare_ct(&self, other: &T) -> bool {
        self.as_slice().ct_eq(other.as_ref()).unwrap_u8() == 1
    }
}

impl<'a> CompareCT<AString<'a>> for AString<'a> {
    fn compare_ct(&self, other: &AString<'a>) -> bool {
        match (self, other) {
            (AString::Atom(lhs), AString::Atom(rhs)) => {
                lhs.as_ref()
                    .as_bytes()
                    .ct_eq(rhs.as_ref().as_bytes())
                    .unwrap_u8()
                    == 1
            }
            (AString::String(lhs), AString::String(rhs)) => lhs.compare_ct(rhs),
            _ => false,
        }
    }
}

impl<'a> CompareCT<IString<'a>> for IString<'a> {
    fn compare_ct(&self, other: &IString<'a>) -> bool {
        match (self, other) {
            (IString::Quoted(lhs), IString::Quoted(rhs)) => {
                lhs.as_ref()
                    .as_bytes()
                    .ct_eq(rhs.as_ref().as_bytes())
                    .unwrap_u8()
                    == 1
            }
            (IString::Literal(lhs), IString::Literal(rhs)) => lhs.compare_ct(rhs),
            _ => false,
        }
    }
}

impl<'a> CompareCT<Literal<'a>> for Literal<'a> {
    fn compare_ct(&self, other: &Literal<'a>) -> bool {
        #[cfg(not(feature = "ext_literal"))]
        return self.as_ref().ct_eq(other.as_ref()).unwrap_u8() == 1;
        #[cfg(feature = "ext_literal")]
        return self.as_ref().ct_eq(other.as_ref()).unwrap_u8() == 1 && self.mode == other.mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ext_literal")]
    use crate::core::Literal;
    use crate::{
        command::{Command, CommandBody},
        core::{AString, Atom, Quoted},
    };

    #[test]
    #[cfg(not(debug_assertions))]
    #[allow(clippy::redundant_clone)]
    fn test_that_secret_is_redacted() {
        #[cfg(feature = "ext_sasl_ir")]
        use crate::auth::AuthMechanism;
        use crate::auth::AuthenticateData;

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
            #[cfg(feature = "ext_sasl_ir")]
            CommandBody::authenticate(AuthMechanism::Plain, Some(b"xyz123"))
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
            AuthenticateData(Secret::new(b"xyz123".to_vec())),
            AuthenticateData(Secret::from(b"xyz123".to_vec())),
        ];

        for test in tests {
            let got = format!("{:?}", test);
            println!("Debug: {}", got);
            assert!(got.contains("/* REDACTED */"));
            assert!(!got.contains("xyz123"));
            assert!(!got.contains("eHl6MTIz"));
        }
    }

    /// A best effort test to ensure that constant-time comparison works.
    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_that_eq_is_constant_time() {
        let took_constant = {
            fn compare_eq(a: Secret<AString>, b: Secret<AString>) -> u128 {
                let tik = std::time::Instant::now();
                assert_eq!(a, b);
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            fn compare_ne(a: Secret<AString>, b: Secret<AString>) -> u128 {
                let tik = std::time::Instant::now();
                assert_ne!(a, b);
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            let a = Secret::new(AString::from(
                Atom::try_from(str::repeat("A", 1024 * 1024)).unwrap(),
            ));
            let b = Secret::new(AString::from(
                Atom::try_from(str::repeat("B", 1024 * 1024)).unwrap(),
            ));

            let took1 = compare_eq(a.clone(), a.clone());
            println!("{}", took1);
            let took2 = compare_ne(a.clone(), b.clone());
            println!("{}", took2);
            let took3 = compare_ne(b.clone(), a.clone());
            println!("{}", took3);
            let took4 = compare_eq(b.clone(), b.clone());
            println!("{}", took4);

            (took1 + took2 + took3 + took4) / 4
        };

        let took_variable = {
            fn compare_eq(a: String, b: String) -> u128 {
                let tik = std::time::Instant::now();
                assert_eq!(a, b);
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            fn compare_ne(a: String, b: String) -> u128 {
                let tik = std::time::Instant::now();
                assert_ne!(a, b);
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            let a = str::repeat("A", 1024 * 1024);
            let b = str::repeat("B", 1024 * 1024);

            let took1 = compare_eq(a.clone(), a.clone());
            println!("{}", took1);
            let took2 = compare_ne(a.clone(), b.clone());
            println!("{}", took2);
            let took3 = compare_ne(b.clone(), a.clone());
            println!("{}", took3);
            let took4 = compare_eq(b.clone(), b.clone());
            println!("{}", took4);

            (took1 + took2 + took3 + took4) / 4
        };

        let times = took_constant / took_variable;
        println!("{took_constant} vs {took_variable} ({times} times slower)");
        if times < 10 {
            panic!("expected slowdown >= 10, got {}", times);
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

        #[cfg(feature = "ext_literal")]
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
