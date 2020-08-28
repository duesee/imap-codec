# IMAP Protocol (Parser and Types)

This library provides
  * Complete parsing and serialization of IMAP commands
  * Semi-complete parsing and serialization of IMAP responses

A goal of this library is to provide a misuse resistent API which makes it hard (or impossible) to construct invalid messages.

If you are working on an IMAP *client*, consider using https://github.com/djc/tokio-imap (imap-proto) first,
as it seems to have better support for responses and received more review.

I am not aware of a more complete implementation of the IMAP server side in Rust. (Please tell me if there is one!)

# Future of this Crate

There is a bunch of crates which all cover a different amount of IMAP. Sadly, this crate is no exception.
I will continue to work on this library, but would also be happy if this could be merged with related IMAP crates to a complete IMAP library.

## Known issues

This library emerged from the need for an IMAP testing tool and some work must be done to further improve the quality of the implementation.

* [ ] remove remaining prototype artifacts like `unwrap()`s and `unimplemented()`s
* [ ] settle on "core types", i.e. decide when to use `&[u8]` or `&str` and when to wrap primitive types in `Atom` or `IMAPString`?
* [ ] exchange `Codec` with something more efficient
* [ ] do not allocate when not needed. Make use of `Cow` (see `quoted` parser).
* [ ] remove irrelevant comments and cite IMAP RFC is an appropriate way.

# Usage notes

Have a look at the `parse_*` examples. You can run it with...

```text
cargo run --example=parse_*
```

Type any valid IMAP4REV1 command and see if you are happy with what you get.

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

The way that IMAP specified literals makes it really hard to separate the parsing logic from the application logic.
When a parsers recognizes a literal (e.g. "{42}"), which can pretty much be used anywhere, a continuation response ("+ ...") must be send.
Otherwise, the client or server won't send any more data and a parser would always return `Incomplete(42)`.

However, we do not want to pass sockets to the parser nor clutter every parser with an `NeedsContinuation` error... (or should we?)
A possible solution right now is to implement a framing codec first, which takes care of sending continuation responses and framing commands and responses.

This strategy is motivated by the IMAP4REV1 RFC:

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

...ignoring the fact, that it *may* be possible for other element to contain a pattern like "{...}\r\n".

Caveat: it is. But apparently only in the `text` parser.

Example:

```
1 OK {5}\r\n
YOLO?\r\n
```

# Status -- which parsers are implemented?

Note that a whole category may still be useful (e.g. base64) even when not every parser is implemented.

## Address

* [x] address
* [x] addr-name
* [x] addr-adl
* [x] addr-mailbox
* [x] addr-host

## Base64

* [x] base64
* [x] base64-char
* [x] base64-terminal (not required)

## Body

* [ ] body
* [ ] body-type-1part
* [ ] body-type-basic
* [ ] body-type-msg
* [ ] body-type-text
* [ ] media-basic
* [ ] media-subtype
* [ ] body-fields
* [ ] body-fld-param
* [ ] body-fld-id
* [ ] body-fld-desc
* [ ] body-fld-enc
* [ ] body-fld-octets
* [ ] body-fld-lines
* [ ] body-ext-1part
* [ ] body-fld-md5
* [ ] body-fld-dsp
* [ ] body-fld-lang
* [ ] body-fld-loc
* [ ] body-extension
* [ ] body-type-mpart
* [ ] body-ext-mpart
* [ ] media-message
* [ ] media-text

# Command

* [x] command
* [x] command-any
* [x] command-auth
* [x] append
* [x] create
* [x] delete
* [x] examine
* [x] list
* [x] lsub
* [x] rename
* [x] select
* [x] status
* [x] subscribe
* [x] unsubscribe
* [x] command-nonauth
* [x] login
* [x] userid
* [x] password
* [x] authenticate
* [x] command-select
* [x] copy
* [x] fetch
* [x] fetch-att
* [x] store
* [x] store-att-flags
* [x] uid
* [x] search
* [x] search-key

# Data Format

## number

* [x] number
* [x] nz-number
* [x] digit-nz

## string

* [x] string
* [x] quoted
* [x] QUOTED-CHAR
* [x] quoted-specials
* [x] literal
* [x] CHAR8

## astring

* [x] astring
* [x] ASTRING-CHAR
* [x] ATOM-CHAR
* [x] resp-specials
* [x] atom

## nstring

* [x] nstring
* [x] nil

# Datetime

* [x] date
* [x] date-text
* [x] date-day
* [x] date-month
* [x] date-year
* [x] time
* [x] date-time
* [x] date-day-fixed
* [x] zone

# Envelope

* [x] envelope
* [x] env-date
* [x] env-subject
* [x] env-from
* [x] env-sender
* [x] env-reply-to
* [x] env-to
* [x] env-cc
* [x] env-bcc
* [x] env-in-reply-to
* [x] env-message-id

# Flag

* [x] flag
* [x] flag-keyword
* [x] flag-extension
* [x] flag-fetch
* [x] flag-list
* [x] flag-perm

# Header

* [x] header-list
* [x] header-fld-name

# Mailbox

* [x] list-mailbox
* [x] list-char
* [x] list-wildcards
* [x] mailbox
* [x] mailbox-data
* [x] mailbox-list
* [x] mbx-list-flags
* [x] mbx-list-oflag
* [x] mbx-list-sflag

# Message

* [x] message-data
* [x] msg-att
* [x] msg-att-dynamic
* [ ] msg-att-static
* [x] uniqueid

# Response

## greeting

* [x] greeting
* [x] resp-cond-auth
* [x] resp-text
* [x] text
* [x] TEXT-CHAR
* [x] resp-text-code
* [x] capability-data
* [x] capability
* [x] resp-cond-bye

## response

* [x] response
* [x] continue-req
* [x] response-data
* [x] resp-cond-state
* [x] response-done
* [x] response-tagged
* [x] response-fatal

# Section

* [x] section
* [ ] section-spec
* [x] section-msgtext
* [x] section-part
* [x] section-text

# Sequence

* [x] sequence-set
* [x] seq-range
* [x] seq-number

# Status

* [x] status-att
* [x] status-att-list
* [x] status-att-val

## Unsorted

* [x] auth-type
* [ ] charset
* [x] tag

# License

This crate is dual-licensed under Apache 2.0 and MIT terms.
