use std::{io::Write, str::from_utf8};

use abnf_core::streaming::dquote;
use imap_types::{
    extensions::utf8::QuotedUtf8,
    utils::{
        escape_quoted,
        indicators::{is_quoted_specials, is_text_char},
        unescape_quoted,
    },
};
use nom::{
    bytes::streaming::{escaped, take_while1},
    character::streaming::one_of,
    combinator::map_res,
    sequence::tuple,
};

use crate::{
    decode::IMAPResult,
    encode::{EncodeContext, EncodeIntoContext},
};

impl EncodeIntoContext for QuotedUtf8<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> std::io::Result<()> {
        write!(ctx, "\"{}\"", escape_quoted(self.inner()))
    }
}

/// [RFC 9755, Section 3](https://www.rfc-editor.org/rfc/rfc9755.html#section-3).
///
/// ```abnf
/// ; QUOTED-CHAR is not modified, as it will affect other RFC 3501 ABNF non-terminals.
/// quoted =/ DQUOTE *uQUOTED-CHAR DQUOTE
///
/// uQUOTED-CHAR  = QUOTED-CHAR / UTF8-2 / UTF8-3 / UTF8-4
/// UTF8-2        =   <Defined in Section 4 of RFC 3629>
/// UTF8-3        =   <Defined in Section 4 of RFC 3629>
/// UTF8-4        =   <Defined in Section 4 of RFC 3629>
/// ```
///
/// This function only allocates a new String, when needed, i.e. when
/// quoted chars need to be replaced.
pub(crate) fn quoted_utf8(input: &[u8]) -> IMAPResult<&[u8], QuotedUtf8> {
    let mut parser = tuple((
        dquote,
        map_res(
            escaped(
                take_while1(|c| (is_text_char(c) || c >= 0x80) && !is_quoted_specials(c)),
                '\\',
                one_of("\\\""),
            ),
            from_utf8,
        ),
        dquote,
    ));

    let (remaining, (_, quoted_utf8, _)) = parser(input)?;

    Ok((
        remaining,
        QuotedUtf8::unvalidated(unescape_quoted(quoted_utf8)),
    ))
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use imap_types::{
        command::CommandBody,
        core::{AString, IString},
        extensions::utf8::QuotedUtf8,
    };

    use super::quoted_utf8;
    use crate::{
        CommandCodec, ResponseCodec,
        decode::{CommandDecodeError, Decoder, ResponseDecodeError},
        testing::{kat_inverse_command, known_answer_test_encode, known_answer_test_parse},
    };

    #[test]
    fn test_quoted_utf8() {
        assert_eq!(
            (b"".as_ref(), QuotedUtf8::try_from("äö¹").unwrap()),
            quoted_utf8("\"äö¹\"".as_bytes()).unwrap()
        );
    }

    #[test]
    fn test_quoted_utf8_edge_cases() {
        for (wire, text) in [
            ("\"\"", ""),
            ("\"café日本語😀\"", "café日本語😀"),
            ("\"café\\\"\\\\\"", "café\"\\"),
            ("\"\t\u{7f}\u{80}\"", "\t\u{7f}\u{80}"),
        ] {
            let wire = wire.as_bytes();
            let expected = QuotedUtf8::try_from(text).unwrap();
            known_answer_test_parse((wire, b"", expected.clone()), quoted_utf8);
            known_answer_test_encode((expected, wire));
            for end in 0..wire.len() {
                assert!(
                    matches!(quoted_utf8(&wire[..end]), Err(nom::Err::Incomplete(_))),
                    "{wire:?}, prefix {end}",
                );
            }
        }
        let (_, decoded) = quoted_utf8("\"café\"".as_bytes()).unwrap();
        assert!(matches!(decoded.into_inner(), Cow::Borrowed("café")));
    }

    #[test]
    fn test_quoted_utf8_invalid() {
        for invalid in [
            b"\0".as_slice(),
            b"\r",
            b"\n",
            b"\xff",
            b"\xed\xa0\x80",
            // RFC 3629, Section 3: overlong NUL.
            b"\xc0\x80",
            b"\xf4\x90\x80\x80",
            b"\xc3",
        ] {
            for prefix in [b"ASCII".as_slice(), "café".as_bytes()] {
                let command = [b"A CREATE \"".as_slice(), prefix, invalid, b"\"\r\n"].concat();
                assert!(
                    matches!(
                        CommandCodec::default().decode(&command),
                        Err(CommandDecodeError::Failed { .. })
                    ),
                    "{command:?}"
                );
                let response =
                    [b"* LIST () \"/\" \"".as_slice(), prefix, invalid, b"\"\r\n"].concat();
                assert!(
                    matches!(
                        ResponseCodec::default().decode(&response),
                        Err(ResponseDecodeError::Failed)
                    ),
                    "{response:?}"
                );
            }
        }
    }

    #[test]
    fn test_kat_inverse_create_utf8() {
        let wire = "A CREATE \"café日本語😀\\\"\\\\\"\r\n".as_bytes();
        let expected = CommandBody::create(AString::String(IString::QuotedUtf8(
            QuotedUtf8::try_from("café日本語😀\"\\").unwrap(),
        )))
        .unwrap()
        .tag("A")
        .unwrap();
        known_answer_test_encode((expected.clone(), wire));
        kat_inverse_command(&[(wire, b"", expected)]);
    }
}
