use imap_types::AuthMechanism;
use nom::IResult;

use crate::rfc3501::core::atom;

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
    let (rem, mechanism) = atom(input)?;

    Ok((rem, mechanism.to_owned().into()))
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use imap_types::AuthMechanism;

    use super::auth_type;

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
