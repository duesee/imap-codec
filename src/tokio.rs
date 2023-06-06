use thiserror::Error;

pub mod client;
pub mod server;

/// All interactions transmitted by client and server are in the form of
/// lines, that is, strings that end with a CRLF.
///
/// The protocol receiver of an IMAP4rev1 client or server is either ...
#[derive(Debug, Clone, PartialEq, Eq)]
enum LiteralFramingState {
    /// ... reading a line, or ...
    ReadLine { to_consume_acc: usize },
    /// ... is reading a sequence of octets
    /// with a known count followed by a line.
    ReadLiteral { to_consume_acc: usize, length: u32 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LineError {
    #[error("Expected `\r\n`, got only `\n`")]
    NotCrLf,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LiteralError {
    #[error("Expected a maximum literal length of {max_length}, got {length}")]
    TooLarge { max_length: u32, length: u32 },
    #[error("Could not parse literal length")]
    BadNumber,
    #[error("Could not find literal length")]
    NoOpeningBrace,
}

fn find_crlf_inclusive(skip: usize, buf: &[u8]) -> Result<Option<usize>, LineError> {
    match buf.iter().skip(skip).position(|item| *item == b'\n') {
        Some(position) => {
            if buf[skip + position.saturating_sub(1)] != b'\r' {
                Err(LineError::NotCrLf)
            } else {
                Ok(Some(position + 1))
            }
        }
        None => Ok(None),
    }
}

fn parse_literal(line: &[u8]) -> Result<Option<u32>, LiteralError> {
    match parse_literal_enclosing(line) {
        Ok(maybe_raw) => {
            if let Some(raw) = maybe_raw {
                let str = std::str::from_utf8(raw).map_err(|_| LiteralError::BadNumber)?;
                let num: u32 = str.parse().map_err(|_| LiteralError::BadNumber)?;

                Ok(Some(num))
            } else {
                Ok(None)
            }
        }
        Err(err) => Err(err),
    }
}

fn parse_literal_enclosing(line: &[u8]) -> Result<Option<&[u8]>, LiteralError> {
    if line.is_empty() {
        return Ok(None);
    }

    if line[line.len() - 1] != b'}' {
        return Ok(None);
    }

    let mut index = line.len() - 1;

    while index > 0 {
        index -= 1;

        if line[index] == b'{' {
            return Ok(Some(&line[index + 1..line.len() - 1]));
        }
    }

    Err(LiteralError::NoOpeningBrace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let tests = [
            (b"".as_ref(), Ok(None)),
            (b"{0}".as_ref(), Ok(Some(0))),
            (b"{123456}".as_ref(), Ok(Some(123456))),
            (b"{4294967295}".as_ref(), Ok(Some(u32::MAX))),
            (b"{4294967296}".as_ref(), Err(LiteralError::BadNumber)),
            (b"{}".as_ref(), Err(LiteralError::BadNumber)),
            (b"{a}".as_ref(), Err(LiteralError::BadNumber)),
            (b"{0a}".as_ref(), Err(LiteralError::BadNumber)),
            (b"{-1}".as_ref(), Err(LiteralError::BadNumber)),
            (b"}".as_ref(), Err(LiteralError::NoOpeningBrace)),
        ];

        for (test, expected) in tests {
            let got = parse_literal(test);
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_parse_literal_enclosing() {
        let tests = [
            (b"".as_ref(), Ok(None)),
            (b"{0}".as_ref(), Ok(Some(b"0".as_ref()))),
            (b"{123456}".as_ref(), Ok(Some(b"123456"))),
            (b"{4294967295}".as_ref(), Ok(Some(b"4294967295"))),
            (b"{4294967296}".as_ref(), Ok(Some(b"4294967296"))),
            (b"{}".as_ref(), Ok(Some(b""))),
            (b"{a}".as_ref(), Ok(Some(b"a"))),
            (b"{0a}".as_ref(), Ok(Some(b"0a"))),
            (b"{-1}".as_ref(), Ok(Some(b"-1"))),
            (b"}".as_ref(), Err(LiteralError::NoOpeningBrace)),
        ];

        for (test, expected) in tests {
            let got = parse_literal_enclosing(test);
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_find_crlf_inclusive() {
        let tests = [
            (b"A\r".as_ref(), 0, Ok(None)),
            (b"A\r\n", 0, Ok(Some(3))),
            (b"A\n", 0, Err(LineError::NotCrLf)),
            (b"\n", 0, Err(LineError::NotCrLf)),
            (b"aaa\r\nA\r".as_ref(), 5, Ok(None)),
            (b"aaa\r\nA\r\n", 5, Ok(Some(3))),
            (b"aaa\r\nA\n", 5, Err(LineError::NotCrLf)),
            (b"aaa\r\n\n", 5, Err(LineError::NotCrLf)),
        ];

        for (test, skip, expected) in tests {
            let got = find_crlf_inclusive(skip, test);

            dbg!((std::str::from_utf8(test).unwrap(), skip, &expected, &got));

            assert_eq!(expected, got);
        }
    }
}
