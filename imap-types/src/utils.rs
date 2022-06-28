use std::borrow::Cow;

// FIXME(extract_types): copied from abnf-core and imap-codec
pub mod indicators {
    /// Any 7-bit US-ASCII character, excluding NUL
    ///
    /// CHAR = %x01-7F
    #[allow(non_snake_case)]
    pub fn is_CHAR(byte: u8) -> bool {
        matches!(byte, 0x01..=0x7f)
    }

    /// Controls
    ///
    /// CTL = %x00-1F / %x7F
    #[allow(non_snake_case)]
    pub fn is_CTL(byte: u8) -> bool {
        matches!(byte, 0x00..=0x1f | 0x7f)
    }

    pub(crate) fn is_any_text_char_except_quoted_specials(byte: u8) -> bool {
        is_text_char(byte) && !is_quoted_specials(byte)
    }

    /// `quoted-specials = DQUOTE / "\"`
    pub fn is_quoted_specials(byte: u8) -> bool {
        byte == b'"' || byte == b'\\'
    }

    /// `ASTRING-CHAR = ATOM-CHAR / resp-specials`
    pub fn is_astring_char(i: u8) -> bool {
        is_atom_char(i) || is_resp_specials(i)
    }

    /// `ATOM-CHAR = <any CHAR except atom-specials>`
    pub fn is_atom_char(b: u8) -> bool {
        is_CHAR(b) && !is_atom_specials(b)
    }

    /// `atom-specials = "(" / ")" / "{" / SP / CTL / list-wildcards / quoted-specials / resp-specials`
    pub fn is_atom_specials(i: u8) -> bool {
        match i {
            b'(' | b')' | b'{' | b' ' => true,
            c if is_CTL(c) => true,
            c if is_list_wildcards(c) => true,
            c if is_quoted_specials(c) => true,
            c if is_resp_specials(c) => true,
            _ => false,
        }
    }

    /// `list-wildcards = "%" / "*"`
    pub fn is_list_wildcards(i: u8) -> bool {
        i == b'%' || i == b'*'
    }

    #[inline]
    /// `resp-specials = "]"`
    pub fn is_resp_specials(i: u8) -> bool {
        i == b']'
    }

    #[inline]
    /// `CHAR8 = %x01-ff`
    ///
    /// Any OCTET except NUL, %x00
    pub fn is_char8(i: u8) -> bool {
        i != 0
    }

    /// `TEXT-CHAR = %x01-09 / %x0B-0C / %x0E-7F`
    ///
    /// Note: This was `<any CHAR except CR and LF>` before.
    pub fn is_text_char(c: u8) -> bool {
        matches!(c, 0x01..=0x09 | 0x0b..=0x0c | 0x0e..=0x7f)
    }

    /// `list-char = ATOM-CHAR / list-wildcards / resp-specials`
    pub fn is_list_char(i: u8) -> bool {
        is_atom_char(i) || is_list_wildcards(i) || is_resp_specials(i)
    }
}

pub fn escape_quoted(unescaped: &str) -> Cow<str> {
    let mut escaped = Cow::Borrowed(unescaped);

    if escaped.contains('\\') {
        escaped = Cow::Owned(escaped.replace('\\', "\\\\"));
    }

    if escaped.contains('\"') {
        escaped = Cow::Owned(escaped.replace('"', "\\\""));
    }

    escaped
}

#[allow(unused)]
pub fn unescape_quoted(escaped: &str) -> Cow<str> {
    let mut unescaped = Cow::Borrowed(escaped);

    if unescaped.contains("\\\\") {
        unescaped = Cow::Owned(unescaped.replace("\\\\", "\\"));
    }

    if unescaped.contains("\\\"") {
        unescaped = Cow::Owned(unescaped.replace("\\\"", "\""));
    }

    unescaped
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_escape_quoted() {
        let tests = [
            ("", ""),
            ("\\", "\\\\"),
            ("\"", "\\\""),
            ("alice", "alice"),
            ("\\alice\\", "\\\\alice\\\\"),
            ("alice\"", "alice\\\""),
            (r#"\alice\ ""#, r#"\\alice\\ \""#),
        ];

        for (test, expected) in tests {
            let got = escape_quoted(test);
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_unescape_quoted() {
        let tests = [
            ("", ""),
            ("\\\\", "\\"),
            ("\\\"", "\""),
            ("alice", "alice"),
            ("\\\\alice\\\\", "\\alice\\"),
            ("alice\\\"", "alice\""),
            (r#"\\alice\\ \""#, r#"\alice\ ""#),
        ];

        for (test, expected) in tests {
            let got = unescape_quoted(test);
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_that_unescape_is_inverse_of_escape() {
        let input = "\\\"\\¹²³abc_*:;059^$%§!\""; // FIXME(#30): use QuickCheck or something similar

        assert_eq!(input, unescape_quoted(escape_quoted(input).as_ref()));
    }
}
