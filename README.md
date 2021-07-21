![CI](https://github.com/duesee/imap-codec/actions/workflows/ci.yml/badge.svg)
![Scheduled](https://github.com/duesee/imap-codec/actions/workflows/scheduled.yml/badge.svg)

# IMAP Protocol

This library provides complete and detailed parsing and construction of [IMAP4rev1](https://tools.ietf.org/html/rfc3501) commands and responses.

The three entry points are `greeting` (to parse the first message from a server), `command` (to parse any message from a client) and `response` (to parse any response or result from a server.) Every parser takes an input (`&[u8]`) and produce a remainder and a parsed value.

# Features

The type-system is used to enforce correctness and make the library misuse resistent. It should not be possible to construct messages, which would violate the IMAP specification.

Fuzzing (via [cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz)) and (soon) property-based tests are used to uncover parsing and serialization bugs. For example, the library is fuzz-tested to never produce a message it can not parse itself. Additionally, many real-world IMAP traces (including all examples and the sample trace from the IMAP RFC) are decoded and encoded correctly.

Every parser works in streaming mode, i.e. all parsers will return `Incomplete` when there is not enough data to make a final decision and no command or response will ever be truncated.

This is (probably) the most complete IMAP implementation in Rust available. Only [tokio-imap](https://github.com/djc/tokio-imap), which you should also check out, provides a comperative amount of features. However, it does not implement the server-side. (Please tell me if there is another one!)

# Usage

```rust
use imap_codec::{
    codec::Encode,           // This trait provides the `serialize` method.
    parse::command::command, // This is the command parser.
};

fn main() {
    let input = b"ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>)\r\n";

    let (_remainder, parsed) = command(input).unwrap();
    println!("// Parsed:");
    println!("{:#?}", parsed);

    let mut serialized = Vec::new();
    parsed.encode(&mut serialized).unwrap(); // This could be send over the network.
    
    let serialized = String::from_utf8(serialized).unwrap(); // Not every IMAP message is valid UTF-8.
    println!("// Serialized:");                              // We just ignore that, so that we can print the message.
    println!("// {}", serialized);
}
```

# Example (binary)

Have a look at the `parse_*` examples and try any IMAP message, e.g.

```
$ cargo run --example=parse_command
```

# Known issues and TODOs

* Serialization is currently done through an ad-hoc `Encoder` trait, which returns an `Vec<u8>`, is super inefficient, and must be phased out.
* Do not allocate when not needed. Make good use of `Cow` (see `quoted` parser).
* The API is still unstable and must be cleaned up.
* Public documentation is still missing.
* Remove irrelevant comments and cite IMAP RFC is an appropriate way
* Decide when `&[u8]` or `&str` is sufficient and when e.g. `Atom` or `IString` are useful.
* Provide "owned" and "referenced" variants of types (work in progress)

## A Note on Allocation and Types

Parsed objects are "owned". This makes them comfortable to use as one has not to think about lifetimes. However, I realize, that a low-level parsing library should be more strict about allocations. Thus, I tried to 1) avoid unnecessary allocations 2) defer allocations as far "up" in the parse tree as possible and 3) always make allocations explicit. In other words: the most low-level parsers do not allocate and allocations are done as late as possible, optimally, right before returning an owned object to the user. This is currently a middle-ground with room for improvement.

Due to the correctness guarantees (and the mentioned allocation strategy), the library uses multiple "string types" like `Atom`, `Tag`, `NString`, and `IString` (in an owned and referenced variant.) I found them quiet useful, but they might not weigh its merit. Positively thinking, this is another opportunity to remove some code.

## A Note on IMAP literals

The way that IMAP specified literals makes it difficult to separate the parsing logic from the application logic. When a parser recognizes a literal (e.g. "{42}"), which can be used anywhere, a so called continuation response ("+ ...") must be send.
Otherwise, the client or server won't send any more data and a parser would always return `Incomplete(42)`.

A possible solution is to implement a "framing codec" first. This strategy is motivated by the IMAP RFC:

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

## Sample IMAP4rev1 connection from RFC 3501

This output was generated by reading the trace from the [IMAP RFC section 8](https://tools.ietf.org/html/rfc3501#section-8), printing the input (first line), printing the Debug of the object (second line), and serializing it again (third line).

```rust
// * OK IMAP4rev1 Service Ready
Status(Ok { tag: None, code: None, text: "IMAP4rev1 Service Ready" })
// * OK IMAP4rev1 Service Ready

// a001 login mrc secret
Command { tag: Tag("a001"), body: Login { username: Atom("mrc"), password: Atom("secret") } }
// a001 LOGIN mrc secret

// a001 OK LOGIN completed
Status(Ok { tag: Some(Tag("a001")), code: None, text: "LOGIN completed" })
// a001 OK LOGIN completed

// a002 select inbox
Command { tag: Tag("a002"), body: Select { mailbox_name: Inbox } }
// a002 SELECT INBOX

// * 18 EXISTS
Data(Exists(18))
// * 18 EXISTS

// * FLAGS (\Answered \Flagged \Deleted \Seen \Draft)
Data(Flags([Answered, Flagged, Deleted, Seen, Draft]))
// * FLAGS (\Answered \Flagged \Deleted \Seen \Draft)

// * 2 RECENT
Data(Recent(2))
// * 2 RECENT

// * OK [UNSEEN 17] Message 17 is the first unseen message
Status(Ok { tag: None, code: Some(Unseen(17)), text: "Message 17 is the first unseen message" })
// * OK [UNSEEN 17] Message 17 is the first unseen message

// * OK [UIDVALIDITY 3857529045] UIDs valid
Status(Ok { tag: None, code: Some(UidValidity(3857529045)), text: "UIDs valid" })
// * OK [UIDVALIDITY 3857529045] UIDs valid

// a002 OK [READ-WRITE] SELECT completed
Status(Ok { tag: Some(Tag("a002")), code: Some(ReadWrite), text: "SELECT completed" })
// a002 OK [READ-WRITE] SELECT completed

// a003 fetch 12 full
Command { tag: Tag("a003"), body: Fetch { sequence_set: [Single(Value(12))], items: Macro(Full), uid: false } }
// a003 FETCH 12 FULL

// * 12 FETCH (FLAGS (\Seen) INTERNALDATE "17-Jul-1996 02:44:25 -0700" RFC822.SIZE 4286 ENVELOPE ("Wed, 17 Jul 1996 02:23:25 -0700 (PDT)" "IMAP4rev1 WG mtg summary and minutes" (("Terry Gray" NIL "gray" "cac.washington.edu")) (("Terry Gray" NIL "gray" "cac.washington.edu")) (("Terry Gray" NIL "gray" "cac.washington.edu")) ((NIL NIL "imap" "cac.washington.edu")) ((NIL NIL "minutes" "CNRI.Reston.VA.US")("John Klensin" NIL "KLENSIN" "MIT.EDU")) NIL NIL "<B27397-0100000@cac.washington.edu>") BODY ("TEXT" "PLAIN" ("CHARSET" "US-ASCII") NIL NIL "7BIT" 3028 92))
Data(Fetch { msg: 12, items: [Flags([Seen]), InternalDate(1996-07-17T02:44:25-07:00), Rfc822Size(4286), Envelope(Envelope { date: NString(Some(Quoted("Wed, 17 Jul 1996 02:23:25 -0700 (PDT)"))), subject: NString(Some(Quoted("IMAP4rev1 WG mtg summary and minutes"))), from: [Address { name: NString(Some(Quoted("Terry Gray"))), adl: NString(None), mailbox: NString(Some(Quoted("gray"))), host: NString(Some(Quoted("cac.washington.edu"))) }], sender: [Address { name: NString(Some(Quoted("Terry Gray"))), adl: NString(None), mailbox: NString(Some(Quoted("gray"))), host: NString(Some(Quoted("cac.washington.edu"))) }], reply_to: [Address { name: NString(Some(Quoted("Terry Gray"))), adl: NString(None), mailbox: NString(Some(Quoted("gray"))), host: NString(Some(Quoted("cac.washington.edu"))) }], to: [Address { name: NString(None), adl: NString(None), mailbox: NString(Some(Quoted("imap"))), host: NString(Some(Quoted("cac.washington.edu"))) }], cc: [Address { name: NString(None), adl: NString(None), mailbox: NString(Some(Quoted("minutes"))), host: NString(Some(Quoted("CNRI.Reston.VA.US"))) }, Address { name: NString(Some(Quoted("John Klensin"))), adl: NString(None), mailbox: NString(Some(Quoted("KLENSIN"))), host: NString(Some(Quoted("MIT.EDU"))) }], bcc: [], in_reply_to: NString(None), message_id: NString(Some(Quoted("<B27397-0100000@cac.washington.edu>"))) }), Body(Single { body: Body { basic: BasicFields { parameter_list: [(Quoted("CHARSET"), Quoted("US-ASCII"))], id: NString(None), description: NString(None), content_transfer_encoding: Quoted("7BIT"), size: 3028 }, specific: Text { subtype: Quoted("PLAIN"), number_of_lines: 92 } }, extension: None })] })
// * 12 FETCH (FLAGS (\Seen) INTERNALDATE "17-Jul-1996 02:44:25 -0700" RFC822.SIZE 4286 ENVELOPE ("Wed, 17 Jul 1996 02:23:25 -0700 (PDT)" "IMAP4rev1 WG mtg summary and minutes" (("Terry Gray" NIL "gray" "cac.washington.edu")) (("Terry Gray" NIL "gray" "cac.washington.edu")) (("Terry Gray" NIL "gray" "cac.washington.edu")) ((NIL NIL "imap" "cac.washington.edu"))((NIL NIL "minutes" "CNRI.Reston.VA.US")("John Klensin" NIL "KLENSIN" "MIT.EDU")) NIL NIL "<B27397-0100000@cac.washington.edu>") BODY ("TEXT" "PLAIN" ("CHARSET" "US-ASCII") NIL NIL "7BIT" 3028 92))

// a003 OK FETCH completed
// Status(Ok { tag: Some(Tag("a003")), code: None, text: "FETCH completed" })
// a003 OK FETCH completed

// a004 fetch 12 body[header]
Command { tag: Tag("a004"), body: Fetch { sequence_set: [Single(Value(12))], items: DataItems([BodyExt { section: Some(Header(None)), partial: None, peek: false }]), uid: false } }
// a004 FETCH 12 BODY[HEADER]

// a004 OK FETCH completed
Status(Ok { tag: Some(Tag("a004")), code: None, text: "FETCH completed" })
// a004 OK FETCH completed

// a005 store 12 +flags \deleted
Command { tag: Tag("a005"), body: Store { sequence_set: [Single(Value(12))], kind: Add, response: Answer, flags: [Deleted], uid: false } }
// a005 STORE 12 +FLAGS (\Deleted)

// * 12 FETCH (FLAGS (\Seen \Deleted))
Data(Fetch { msg: 12, items: [Flags([Seen, Deleted])] })
// * 12 FETCH (FLAGS (\Seen \Deleted))

// a005 OK +FLAGS completed
Status(Ok { tag: Some(Tag("a005")), code: None, text: "+FLAGS completed" })
// a005 OK +FLAGS completed

// a006 logout
Command { tag: Tag("a006"), body: Logout }
// a006 LOGOUT

// * BYE IMAP4rev1 server terminating connection
Status(Bye { code: None, text: "IMAP4rev1 server terminating connection" })
// * BYE IMAP4rev1 server terminating connection

// a006 OK LOGOUT completed
Status(Ok { tag: Some(Tag("a006")), code: None, text: "LOGOUT completed" })
// a006 OK LOGOUT completed 
```

# License

This crate is dual-licensed under Apache 2.0 and MIT terms.
