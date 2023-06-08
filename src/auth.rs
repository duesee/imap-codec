use abnf_core::streaming::crlf as CRLF;
/// Re-export everything from imap-types.
pub use imap_types::auth::*;
use nom::{combinator::map, sequence::terminated};

use crate::{
    codec::IMAPResult,
    core::{atom, base64},
    secret::Secret,
};

// ----- Unsorted IMAP parsers -----

/// `auth-type = atom`
///
/// Note: Defined by \[SASL\]
pub fn auth_type(input: &[u8]) -> IMAPResult<&[u8], AuthMechanism> {
    let (rem, atom) = atom(input)?;

    Ok((rem, AuthMechanism::from(atom)))
}

/// `authenticate = "AUTHENTICATE" SP auth-type *(CRLF base64)` (edited)
///
/// ```text
/// authenticate = base64 CRLF
///                vvvvvvvvvvvv
///                |
///                This is parsed here.
///                CRLF is additionally parsed in this parser.
///                FIXME: Multiline base64 currently does not work.
/// ```
pub fn authenticate_data(input: &[u8]) -> IMAPResult<&[u8], AuthenticateData> {
    map(terminated(base64, CRLF), |data| {
        AuthenticateData(Secret::new(data))
    })(input) // FIXME: many0 deleted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{known_answer_test_encode, known_answer_test_parse};

    #[test]
    fn test_encode_auth_mechanism() {
        let tests = [
            (AuthMechanism::Plain, b"PLAIN".as_ref()),
            (AuthMechanism::Login, b"LOGIN"),
            (
                AuthMechanism::Other(AuthMechanismOther::try_from("PLAINX").unwrap()),
                b"PLAINX",
            ),
            (
                AuthMechanism::Other(AuthMechanismOther::try_from("LOGINX").unwrap()),
                b"LOGINX",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_parse_auth_type() {
        let tests = [
            (b"plain ".as_ref(), b" ".as_ref(), AuthMechanism::Plain),
            (b"pLaiN ", b" ", AuthMechanism::Plain),
            (b"lOgiN ", b" ", AuthMechanism::Login),
            (b"login ", b" ", AuthMechanism::Login),
            (b"loginX ", b" ", AuthMechanism::try_from("loginX").unwrap()),
            (
                b"loginX ",
                b" ",
                AuthMechanism::Other(AuthMechanismOther::try_from(b"loginX".as_ref()).unwrap()),
            ),
            (b"Xplain ", b" ", AuthMechanism::try_from("Xplain").unwrap()),
            (
                b"Xplain ",
                b" ",
                AuthMechanism::Other(AuthMechanismOther::try_from(b"Xplain".as_ref()).unwrap()),
            ),
        ];

        for test in tests {
            known_answer_test_parse(test, auth_type);
        }
    }
}
