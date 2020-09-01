# IMAP Protocol

This library provides complete and *detailed* parsing of all IMAP4rev1 commands and responses.
Every command (and most responses) can be constructed and serialized.

Every parser works in streaming mode, i.e. all parsers will return `Incomplete` when there is not enough data to make a final decision and no command or response will ever be truncated.

The two entry points are `command(input: &[u8])` and `response(input: &[u8])`.
Both parsers are regularily tested against [cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer) and, as of now, did not reveal a single crash/panic/stack overflow. Although, the fuzzing process helped to find serialization mistakes.

A goal of this library is to provide a misuse resistent API which makes it hard (or impossible) to construct invalid messages.
Thus, the newtype pattern is used often. However, it should be easy to construct types using `From<&str>`, and, in case not every string reflects the constraints of the type, `TryFrom<&str>`.

# Alternatives

If you are working on an IMAP *client*, you should also consider https://github.com/djc/tokio-imap (imap-proto), as it received more review.

I am not aware of a more complete implementation of IMAP in Rust. (Please tell me if there is one!)

# Future of this Crate

There is a bunch of crates which all cover a different amount of IMAP. Sadly, this crate is no exception.
I will continue to work on this library, but would also be happy if this could be merged with related IMAP crates to a complete IMAP library.

## Known issues and TODOs

This library emerged from the need for an IMAP testing tool and some work must be done to further improve the quality of the implementation.

* [ ] implement missing serialization of `Body`
* [ ] decide when `&[u8]` or `&str` is sufficient and when e.g. `Atom` or `IString` are beneficial
* [ ] provide "owned" and "referenced" variants of types (work in progress)
* [ ] switch from naive `Codec` to something more efficient/standard
* [ ] do not allocate when not needed. Make use of `Cow` (see `quoted` parser)
* [ ] remove irrelevant comments and cite IMAP RFC is an appropriate way

# Usage notes

Have a look at the `parse_*` examples. You can run it with...

```text
$ cargo run --example=parse_*
```

Type any valid IMAP4REV1 command and see if you are happy with what you get.

# Example

```sh
$ cargo run --example=parse_command
```
```rust
Enter IMAP4REV1 command (or "exit"): ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>) 
Command {
    tag: "ABCD",
    body: Fetch {
        sequence_set: [
            Single(
                Value(
                    1,
                ),
            ),
            Range(
                Value(
                    2,
                ),
                Unlimited,
            ),
        ],
        items: DataItems(
            [
                BodyExt {
                    section: Some(
                        Mime(
                            Part(
                                [
                                    1,
                                    2,
                                    3,
                                    4,
                                ],
                            ),
                        ),
                    ),
                    partial: Some(
                        (
                            42,
                            1337,
                        ),
                    ),
                    peek: true,
                },
            ],
        ),
        uid: true,
    },
}
```

# Example (trace from the IMAP RFC)

```rust
// * OK IMAP4rev1 Service Ready\r
Status(Ok { tag: None, code: None, text: "IMAP4rev1 Service Ready" })

// a001 login mrc secret\r
Command { tag: "a001", body: Login { username: Atom("mrc"), password: Atom("secret") } }

// a001 OK LOGIN completed\r
Status(Ok { tag: Some("a001"), code: None, text: "LOGIN completed" })

// a002 select inbox\r
Command { tag: "a002", body: Select(Inbox) }

// * 18 EXISTS\r
Data(Exists(18))

// * FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)\r
Data(Flags([Answered, Flagged, Deleted, Seen, Draft]))

// * 2 RECENT\r
Data(Recent(2))

// * OK [UNSEEN 17] Message 17 is the first unseen message\r
Status(Ok { tag: None, code: Some(Unseen(17)), text: "Message 17 is the first unseen message" })

// * OK [UIDVALIDITY 3857529045] UIDs valid\r
Status(Ok { tag: None, code: Some(UidValidity(3857529045)), text: "UIDs valid" })

// a002 OK [READ-WRITE] SELECT completed\r
Status(Ok { tag: Some("a002"), code: Some(ReadWrite), text: "SELECT completed" })

// a003 fetch 12 full\r
Command { tag: "a003", body: Fetch { sequence_set: [Single(Value(12))], items: Macro(Full) } }

// * 12 FETCH (FLAGS (\\Seen) INTERNALDATE "17-Jul-1996 02:44:25 -0700")\r
Data(Fetch(12, [Flags([Seen]), InternalDate(1996-07-16T19:44:25-07:00)]))

// a003 OK FETCH completed\r
Status(Ok { tag: Some("a003"), code: None, text: "FETCH completed" })

// a004 fetch 12 body[header]\r
Command { tag: "a004", body: Fetch { sequence_set: [Single(Value(12))], items: DataItems([BodyExt { section: Some(Header(None)), partial: None, peek: false }]) } }

// * 12 FETCH (BODY[HEADER] {3}\r
XXX)\r
Data(Fetch(12, [BodyExt { section: Some(Header(None)), origin: None, data: String(Literal([88, 88, 88])) }]))

// a004 OK FETCH completed\r
Status(Ok { tag: Some("a004"), code: None, text: "FETCH completed" })

// a005 store 12 +flags \\deleted\r
Command { tag: "a005", body: Store { sequence_set: [Single(Value(12))], kind: Add, response: Answer, flags: [Deleted] } }

// * 12 FETCH (FLAGS (\\Seen \\Deleted))\r
Data(Fetch(12, [Flags([Seen, Deleted])]))

// a005 OK +FLAGS completed\r
Status(Ok { tag: Some("a005"), code: None, text: "+FLAGS completed" })

// a006 logout\r
Command { tag: "a006", body: Logout }

// * BYE IMAP4rev1 server terminating connection\r
Status(Bye { code: None, text: "IMAP4rev1 server terminating connection" })

// a006 OK LOGOUT completed\r
Status(Ok { tag: Some("a006"), code: None, text: "LOGOUT completed" })
```

## IMAP literals

The way that IMAP specified literals makes it difficult to separate the parsing logic from the application logic.
When a parsers recognizes a literal (e.g. "{42}"), which can pretty much be used anywhere, a continuation response ("+ ...") must be send.
Otherwise, the client or server won't send any more data and a parser would always return `Incomplete(42)`.

However, we do not want to pass sockets to the parser nor clutter every parser with an `NeedsContinuation` error...
A possible solution is to implement a "framing codec" first. This strategy is motivated by the IMAP4REV1 RFC:

```
The protocol receiver of an IMAP4rev1 client or server is either reading a line,
or is reading a sequence of octets with a known count followed by a line.
```

Thus, the framing codec may be implemented like this...

```
loop {
    line = read_line()
    if line.has_literal() {
        literal = read_literal(amount)
    }
}
```

# Status

The complete [formal syntax](https://tools.ietf.org/html/rfc3501#section-9) of IMPA4rev1 is implemented.

# License

This crate is dual-licensed under Apache 2.0 and MIT terms.
