use std::borrow::Cow;

#[cfg(not(feature = "quirk_crlf_relaxed"))]
use abnf_core::streaming::crlf;
#[cfg(feature = "quirk_crlf_relaxed")]
use abnf_core::streaming::crlf_relaxed as crlf;
use imap_types::{core::LiteralMode, extensions::binary::Literal8};
use nom::{
    bytes::streaming::{tag, take},
    character::streaming::char,
    combinator::{map, opt},
    sequence::{delimited, terminated, tuple},
};

use crate::{
    core::number,
    decode::{IMAPErrorKind, IMAPParseError, IMAPResult},
};

#[allow(unused)] // TODO(444)
/// See https://datatracker.ietf.org/doc/html/rfc3516 and https://datatracker.ietf.org/doc/html/rfc4466
///
/// ```abnf
/// literal8 = "~{" number ["+"] "}" CRLF *OCTET
/// ;; <number> represents the number of OCTETs in the response string.
/// ;; The "+" is only allowed when both LITERAL+ and BINARY extensions are supported by the server.
/// ```
pub(crate) fn literal8(input: &[u8]) -> IMAPResult<&[u8], Literal8> {
    let (remaining, (length, mode)) = terminated(
        delimited(
            tag(b"~{"),
            tuple((
                number,
                map(opt(char('+')), |i| {
                    i.map(|_| LiteralMode::NonSync).unwrap_or(LiteralMode::Sync)
                }),
            )),
            tag(b"}"),
        ),
        crlf,
    )(input)?;

    // Signal that an continuation request could be required.
    // Note: This doesn't trigger when there is data following the literal prefix.
    if remaining.is_empty() {
        return Err(nom::Err::Failure(IMAPParseError {
            input,
            kind: IMAPErrorKind::Literal {
                // We don't know the tag here and rely on an upper parser, e.g., `command` to fill this in.
                tag: None,
                length,
                mode,
            },
        }));
    }

    let (remaining, data) = take(length)(remaining)?;

    Ok((
        remaining,
        Literal8 {
            data: Cow::Borrowed(data),
            mode,
        },
    ))
}
