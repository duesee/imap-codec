use std::fmt::{Debug, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
// Note: The implementation of these traits does agree:
//       `PartialEq` is just a thin wrapper that ensures constant-time comparison.
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Hash)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn expose_secret(&self) -> &T {
        &self.0
    }
}

impl<S> Secret<S>
where
    S: AsRef<[u8]>,
{
    pub fn compare_ct<O>(&self, other: O) -> bool
    where
        O: AsRef<[u8]>,
    {
        self.0.as_ref().ct_eq(other.as_ref()).unwrap_u8() == 1
    }
}

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "/* REDACTED */")
    }
}

impl<T> Eq for Secret<T> where T: AsRef<[u8]> {}

impl<T> PartialEq for Secret<T>
where
    T: AsRef<[u8]>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().ct_eq(other.0.as_ref()).unwrap_u8() == 1
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ext_literal")]
    use crate::message::AuthMechanism;
    use crate::{
        codec::Encode,
        command::{AuthenticateData, CommandBody},
        security::Secret,
    };

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_that_secret_is_redacted() {
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
            println!(
                "Serialized: {:?}",
                String::from_utf8(test.encode_detached().unwrap()),
            );

            let got = format!("{:?}", test);
            println!("Debug: {}", got);
            assert!(got.contains("/* REDACTED */"));
            assert!(!got.contains("xyz123"));
            assert!(!got.contains("eHl6MTIz"));

            println!();
        }

        println!("-----");

        let test = AuthenticateData(Secret::new(b"xyz123".to_vec()));
        println!(
            "Serialized: {:?}",
            String::from_utf8(test.encode_detached().unwrap()),
        );

        let got = format!("{:?}", test);
        println!("Debug: {}", got);
        assert!(got.contains("/* REDACTED */"));
        assert!(!got.contains("xyz123"));
        assert!(!got.contains("eHl6MTIz"));
    }

    /// A best effort test to ensure that constant-time comparison works.
    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_that_eq_is_constant_time() {
        let took_constant = {
            fn compare_eq(a: Secret<String>, b: Secret<String>) -> u128 {
                let tik = std::time::Instant::now();
                assert_eq!(a, b);
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            fn compare_ne(a: Secret<String>, b: Secret<String>) -> u128 {
                let tik = std::time::Instant::now();
                assert_ne!(a, b);
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            let a = Secret::new(str::repeat("A", 1024 * 1024));
            let b = Secret::new(str::repeat("B", 1024 * 1024));

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
        assert!(times > 100);
    }

    /// A best effort test to ensure that constant-time comparison works.
    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_that_compare_ct_is_constant_time() {
        let took_constant = {
            fn compare_eq(a: Secret<String>, b: Secret<String>) -> u128 {
                let tik = std::time::Instant::now();
                assert!(a.compare_ct(b.expose_secret()));
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            fn compare_ne(a: Secret<String>, b: Secret<String>) -> u128 {
                let tik = std::time::Instant::now();
                assert!(!a.compare_ct(b.expose_secret()));
                let tok = std::time::Instant::now();

                tok.duration_since(tik).as_nanos()
            }

            let a = Secret::new(str::repeat("A", 1024 * 1024));
            let b = Secret::new(str::repeat("B", 1024 * 1024));

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
        assert!(times > 100);
    }
}
