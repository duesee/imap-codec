use imap_types::message::AuthMechanism;
use nom::IResult;

use crate::imap4rev1::core::atom;

pub mod address;
pub mod body;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
pub mod fetch_attributes;
pub mod flag;
pub mod mailbox;
pub mod response;
pub mod section;
pub mod sequence;
pub mod status_attributes;

// ----- Unsorted IMAP parsers -----

/// `auth-type = atom`
///
/// Note: Defined by [SASL]
pub fn auth_type(input: &[u8]) -> IResult<&[u8], AuthMechanism> {
    let (rem, atom) = atom(input)?;

    Ok((rem, AuthMechanism::from(atom)))
}

#[cfg(test)]
mod tests {
    use imap_types::message::{AuthMechanism, AuthMechanismOther};

    use super::auth_type;
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
