# IMAP Server Protocol (Parser and Types)

This library implements the parsing of IMAP commands (no responses) and the construction and serialization of IMAP responses.
It is thus only useful when implementing an IMAP *server*.

If you are working on an IMAP *client*, consider using https://github.com/djc/tokio-imap (imap-proto).

All types in this crate should be designed in a way which makes it hard (or impossible) to construct invalid messages.
I am not aware of a more complete implementation of the IMAP server side in Rust. (Please tell me if there is one!)

# Future of this Crate

There is a bunch of crates which all cover a different amount of IMAP. Sadly, this crate is no exception.
Ideally, I would like to see this merged in a generic IMAP library. This is also why this is not published on crates.io.

## Known issues

Some work must be done to improve the quality of this implementation.

* [ ] remove prototype artifacts like `unwrap()`s and `unimplemented()`s
* [ ] do not allocate when not needed. Make good use of `Cow`.
* [ ] settle on "core types", i.e. when to use `&[u8]` and `&str` and when to wrap primitive types, e.g. in an `Atom` or `IMAPString`?
* [ ] ...

# Documentation

This project started by copy-pasting the full IMAP RFC in a lib.rs file. This helped me in understanding the protocol.
However, a lot of comments are irrelevant and should be removed. The IMAP RFC should also be cited in an appropriate way.

# Usage notes

Have a look at the `parse_command` example. You can run it with...

```text
cargo run --example=parse_command
```

Type any valid IMAP4REV1 command and see what you get.

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
* [ ] base64-terminal

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
* [ ] QUOTED-CHAR
* [x] quoted-specials
* [ ] literal
* [ ] CHAR8

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

* [ ] envelope
* [x] env-date
* [x] env-subject
* [ ] env-from
* [ ] env-sender
* [ ] env-reply-to
* [ ] env-to
* [ ] env-cc
* [ ] env-bcc
* [x] env-in-reply-to
* [x] env-message-id

# Flag

* [x] flag
* [x] flag-keyword
* [x] flag-extension
* [ ] flag-fetch
* [x] flag-list
* [ ] flag-perm

# Header

* [x] header-list
* [x] header-fld-name

# Mailbox

* [x] list-mailbox
* [x] list-char
* [x] list-wildcards

* [ ] mailbox
* [ ] mailbox-data
* [ ] mailbox-list
* [ ] mbx-list-flags
* [ ] mbx-list-oflag
* [ ] mbx-list-sflag

# Message

* [ ] message-data
* [ ] msg-att
* [ ] msg-att-dynamic
* [ ] msg-att-static
* [x] uniqueid

# Response

## greeting

* [ ] greeting
* [ ] resp-cond-auth
* [x] resp-text
* [x] text
* [x] TEXT-CHAR
* [ ] resp-text-code
* [ ] capability-data
* [ ] capability
* [ ] resp-cond-bye

## response

* [ ] response
* [x] continue-req
* [ ] response-data
* [ ] resp-cond-state
* [ ] response-done
* [ ] response-tagged
* [ ] response-fatal

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
