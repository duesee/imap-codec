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
