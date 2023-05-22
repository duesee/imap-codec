use std::io::Write;

use imap_types::extensions::literal::LiteralCapability;

use crate::codec::Encode;

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
    use super::*;
    use crate::{core::Literal, testing::known_answer_test_encode};

    #[test]
    fn test_encode_literal_capability() {
        let tests = [
            (LiteralCapability::Plus, b"LITERAL+".as_ref()),
            (LiteralCapability::Minus, b"LITERAL-"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_encode_literal_plus() {
        let tests = [
            (
                Literal::try_from("ABCDE").unwrap(),
                b"{5}\r\nABCDE".to_vec(),
            ),
            (
                Literal::try_from("ABCDE").unwrap().into_sync(),
                b"{5}\r\nABCDE".to_vec(),
            ),
            (
                Literal::try_from("ABCDE").unwrap().into_non_sync(),
                b"{5+}\r\nABCDE".to_vec(),
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
