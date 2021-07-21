use std::{borrow::Cow, io::Write, iter};

use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::{codec::Encode, types::core::Tag};

pub(crate) fn gen_tag() -> Tag {
    let mut rng = thread_rng();
    let tag: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(8)
        .collect();

    Tag(tag)
}

pub(crate) fn join<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
    elements
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(sep)
}

pub(crate) fn join_serializable<I: Encode>(
    elements: &[I],
    sep: &[u8],
    writer: &mut impl Write,
) -> std::io::Result<()> {
    if let Some((last, head)) = elements.split_last() {
        for item in head {
            item.encode(writer)?;
            writer.write_all(sep)?;
        }

        last.encode(writer)
    } else {
        Ok(())
    }
}

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
