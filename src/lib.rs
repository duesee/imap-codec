pub mod codec;
pub mod parse;
pub mod state;
pub mod types;
pub mod utils;

#[cfg(test)]
mod test {
    use crate::parse::{command::command, response::response};
    use nom::AsBytes;

    fn escape(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|byte| match byte {
                0x00..=0x08 => format!("\\x{:02x}", byte),
                0x09 => String::from("\\t"),
                0x0A => String::from("\\n\n"),
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

    #[test]
    fn test_transcript_from_rfc() {
        let transcript = [
            ('S', b"* OK IMAP4rev1 Service Ready\r\n".as_bytes()),
            ('C', b"a001 login mrc secret\r\n"),
            ('S', b"a001 OK LOGIN completed\r\n"),
            ('C', b"a002 select inbox\r\n"),
            ('S', b"* 18 EXISTS\r\n"),
            (
                'S',
                b"* FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)\r\n",
            ),
            ('S', b"* 2 RECENT\r\n"),
            (
                'S',
                b"* OK [UNSEEN 17] Message 17 is the first unseen message\r\n",
            ),
            ('S', b"* OK [UIDVALIDITY 3857529045] UIDs valid\r\n"),
            ('S', b"a002 OK [READ-WRITE] SELECT completed\r\n"),
            ('C', b"a003 fetch 12 full\r\n"),
            (
                'S',
                b"* 12 FETCH (FLAGS (\\Seen) INTERNALDATE \"17-Jul-1996 02:44:25 -0700\")\r\n", // shortened...
            ),
            ('S', b"a003 OK FETCH completed\r\n"),
            ('C', b"a004 fetch 12 body[header]\r\n"),
            (
                'S',
                b"* 12 FETCH (BODY[HEADER] {3}\r\nXXX)\r\n", // shortened...
            ),
            ('S', b"a004 OK FETCH completed\r\n"),
            ('C', b"a005 store 12 +flags \\deleted\r\n"),
            ('S', b"* 12 FETCH (FLAGS (\\Seen \\Deleted))\r\n"),
            ('S', b"a005 OK +FLAGS completed\r\n"),
            ('C', b"a006 logout\r\n"),
            ('S', b"* BYE IMAP4rev1 server terminating connection\r\n"),
            ('S', b"a006 OK LOGOUT completed\r\n"),
        ];

        for (side, test) in transcript.iter() {
            match side {
                'C' => {
                    let (_rem, cmd) = command(test).unwrap();
                    println!("// {}", escape(test).trim());
                    println!("{:?}\n", cmd);
                }
                'S' => {
                    // FIXME: many response parsers are not implemented yet. Activate this test later.
                    let (_rem, rsp) = response(test).unwrap();
                    println!("// {}", escape(test).trim());
                    println!("{:?}\n", rsp);
                }
                _ => unreachable!(),
            };
        }
    }
}
