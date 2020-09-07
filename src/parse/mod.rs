use crate::{parse::core::atom, types::AuthMechanism};
use nom::IResult;

pub mod address;
pub mod body;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
pub mod flag;
pub mod header;
pub mod mailbox;
pub mod message;
pub mod response;
pub mod section;
pub mod sequence;
pub mod status;

// ----- Unsorted IMAP parsers -----

/// auth-type = atom
///
/// Note: Defined by [SASL]
pub(crate) fn auth_type(input: &[u8]) -> IResult<&[u8], AuthMechanism> {
    let (rem, raw_mechanism) = atom(input)?;

    // FIXME: just take inner String?
    let mechanism = match raw_mechanism.0.to_lowercase().as_ref() {
        "plain" => AuthMechanism::Plain,
        "login" => AuthMechanism::Login,
        _ => AuthMechanism::Other(raw_mechanism.to_owned()),
    };

    Ok((rem, mechanism))
}

#[cfg(test)]
mod test {
    use super::auth_type;
    use crate::types::AuthMechanism;
    use std::convert::TryInto;

    #[test]
    fn test_auth_type() {
        let tests = [
            (b"plain ".as_ref(), AuthMechanism::Plain),
            (b"pLaiN ".as_ref(), AuthMechanism::Plain),
            (b"lOgiN ".as_ref(), AuthMechanism::Login),
            (b"login ".as_ref(), AuthMechanism::Login),
            (
                b"loginX ".as_ref(),
                AuthMechanism::Other("loginX".try_into().unwrap()),
            ),
            (
                b"Xplain ".as_ref(),
                AuthMechanism::Other("Xplain".try_into().unwrap()),
            ),
        ];

        for (test, expected) in tests.iter() {
            let (rem, got) = auth_type(test).unwrap();
            assert_eq!(*expected, got);
            assert_eq!(rem, b" ");
        }
    }
}
