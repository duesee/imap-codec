use std::borrow::Cow;

pub fn escape_byte_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| match byte {
            0x00..=0x08 => format!("\\x{:02x}", byte),
            0x09 => String::from("\\t"),
            0x0A => String::from("\\n"),
            0x0B => format!("\\x{:02x}", byte),
            0x0C => format!("\\x{:02x}", byte),
            0x0D => String::from("\\r"),
            0x0e..=0x1f => format!("\\x{:02x}", byte),
            0x20..=0x22 => format!("{}", *byte as char),
            0x23..=0x5B => format!("{}", *byte as char),
            0x5C => String::from("\\\\"),
            0x5D..=0x7E => format!("{}", *byte as char),
            0x7f => format!("\\x{:02x}", byte),
            0x80..=0xff => format!("\\x{:02x}", byte),
        })
        .collect::<Vec<String>>()
        .join("")
}

#[allow(unused)]
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
