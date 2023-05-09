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
            0x20..=0x21 => format!("{}", *byte as char),
            0x22 => String::from("\\\""),
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
mod tests {
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
        let input = "\\\"\\¹²³abc_*:;059^$%§!\"";

        assert_eq!(input, unescape_quoted(escape_quoted(input).as_ref()));
    }

    #[test]
    fn test_escape_byte_string() {
        for byte in 0u8..=255 {
            let got = escape_byte_string(&[byte]);

            if byte.is_ascii_alphanumeric() {
                assert_eq!((byte as char).to_string(), got.to_string());
            } else if byte.is_ascii_whitespace() {
                if byte == b'\t' {
                    assert_eq!(String::from("\\t"), got);
                } else if byte == b'\n' {
                    assert_eq!(String::from("\\n"), got);
                }
            } else if byte.is_ascii_punctuation() {
                if byte == b'\\' {
                    assert_eq!(String::from("\\\\"), got);
                } else if byte == b'"' {
                    assert_eq!(String::from("\\\""), got);
                } else {
                    assert_eq!((byte as char).to_string(), got);
                }
            } else {
                assert_eq!(format!("\\x{:02x}", byte), got);
            }
        }

        let tests = [(b"Hallo \"\\\x00", String::from(r#"Hallo \"\\\x00"#))];

        for (test, expected) in tests {
            let got = escape_byte_string(test);
            assert_eq!(expected, got);
        }
    }
}
