use std::borrow::Cow;

pub fn escape_quoted(unescaped: &str) -> Cow<str> {
    let mut escaped = Cow::Borrowed(unescaped);

    if escaped.contains('\\') {
        escaped = Cow::Owned(escaped.replace("\\", "\\\\"));
    }

    if escaped.contains('\"') {
        escaped = Cow::Owned(escaped.replace("\"", "\\\""));
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
        assert_eq!(escape_quoted("alice"), "alice");
        assert_eq!(escape_quoted("\\alice\\"), "\\\\alice\\\\");
        assert_eq!(escape_quoted("alice\""), "alice\\\"");
        assert_eq!(escape_quoted(r#"\alice\ ""#), r#"\\alice\\ \""#);
    }

    #[test]
    fn test_unescape_quoted() {
        assert_eq!(unescape_quoted("alice"), "alice");
        assert_eq!(unescape_quoted("\\\\alice\\\\"), "\\alice\\");
        assert_eq!(unescape_quoted("alice\\\""), "alice\"");
        assert_eq!(unescape_quoted(r#"\\alice\\ \""#), r#"\alice\ ""#);
    }
}
