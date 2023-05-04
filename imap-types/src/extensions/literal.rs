//! IMAP4 Non-synchronizing Literals

use std::io::Write;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::codec::Encode;

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralCapability {
    Plus,
    Minus,
}

impl Encode for LiteralCapability {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Plus => writer.write_all(b"LITERAL+"),
            Self::Minus => writer.write_all(b"LITERAL-"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use crate::{codec::Encode, core::Literal};

    #[test]
    fn test_encoding_of_literal_capability() {
        let tests = [
            (LiteralCapability::Plus, b"LITERAL+".as_ref()),
            (LiteralCapability::Minus, b"LITERAL-"),
        ];

        for (test, expected) in tests {
            let got = test.encode_detached().unwrap();
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_literal_plus() {
        let tests = vec![
            (
                b"{5}\r\nABCDE".to_vec(),
                Literal::try_from("ABCDE").unwrap(),
            ),
            (
                b"{5}\r\nABCDE".to_vec(),
                Literal::try_from("ABCDE").unwrap().into_sync(),
            ),
            (
                b"{5+}\r\nABCDE".to_vec(),
                Literal::try_from("ABCDE").unwrap().into_non_sync(),
            ),
        ];

        for (expected, test) in tests.into_iter() {
            let got = test.encode_detached().unwrap();
            assert_eq!(expected, got);
        }
    }
}
