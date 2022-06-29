pub mod client;
pub mod server;

use bytes::BytesMut;

/// All interactions transmitted by client and server are in the form of
/// lines, that is, strings that end with a CRLF.
///
/// The protocol receiver of an IMAP4rev1 client or server is either ...
#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    /// ... reading a line, or ...
    ReadLine { to_consume_acc: usize },
    /// ... is reading a sequence of octets
    /// with a known count followed by a line.
    ReadLiteral { to_consume_acc: usize, needed: u32 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum LineKind {
    NotCrLf,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LiteralKind {
    TooLarge(u32),
    BadNumber,
    NoOpeningBrace,
}

fn find_crlf_inclusive(skip: usize, buf: &BytesMut) -> Result<Option<usize>, LineKind> {
    match buf.iter().skip(skip).position(|item| *item == b'\n') {
        Some(position) => {
            if buf[skip + position.saturating_sub(1)] != b'\r' {
                Err(LineKind::NotCrLf)
            } else {
                Ok(Some(position + 1))
            }
        }
        None => Ok(None),
    }
}

fn parse_literal(line: &[u8]) -> Result<Option<u32>, LiteralKind> {
    match parse_literal_enclosing(line) {
        Ok(maybe_raw) => {
            if let Some(raw) = maybe_raw {
                let str = std::str::from_utf8(raw).map_err(|_| LiteralKind::BadNumber)?;
                let num = u32::from_str_radix(str, 10).map_err(|_| LiteralKind::BadNumber)?;

                Ok(Some(num))
            } else {
                Ok(None)
            }
        }
        Err(err) => Err(err),
    }
}

fn parse_literal_enclosing(line: &[u8]) -> Result<Option<&[u8]>, LiteralKind> {
    if line.len() == 0 {
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

    return Err(LiteralKind::NoOpeningBrace);
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_find_crlf_inclusive() {
        let tests = [
            (b"A\r".as_ref(), 0, Ok(None)),
            (b"A\r\n", 0, Ok(Some(3))),
            (b"A\n", 0, Err(LineKind::NotCrLf)),
            (b"\n", 0, Err(LineKind::NotCrLf)),
            (b"aaa\r\nA\r".as_ref(), 5, Ok(None)),
            (b"aaa\r\nA\r\n", 5, Ok(Some(3))),
            (b"aaa\r\nA\n", 5, Err(LineKind::NotCrLf)),
            (b"aaa\r\n\n", 5, Err(LineKind::NotCrLf)),
        ];

        for (test, skip, expected) in tests {
            let bytes = BytesMut::from(test);

            let got = find_crlf_inclusive(skip, &bytes);

            dbg!((std::str::from_utf8(test).unwrap(), skip, &expected, &got));

            assert_eq!(expected, got);
        }
    }
}
