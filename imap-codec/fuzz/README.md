# Fuzzing

## Setup

Cargo fuzz requires nightly and `cargo-fuzz`. Install via ...

```sh
rustup install nightly
cargo install cargo-fuzz
```

## Provided fuzz targets

You can start the fuzzing process by running ...

```sh
cargo +nightly fuzz run <target>
```

... with `<target>` being ...

| Name of the fuzz target `<target>`    | Purpose                | Expectation    |
|---------------------------------------|------------------------|----------------|
| `greeting`                            | Test parsing           | Must not fail. |
| `command`                             | Test parsing           | Must not fail. |
| `response`                            | Test parsing           | Must not fail. |
| `authenticate_data`                   | Test parsing           | Must not fail. |
| `idle_done`                           | Test parsing           | Must not fail. |
| `greeting_to_bytes_and_back`          | Test misuse-resistance | Must not fail. |
| `command_to_bytes_and_back`           | Test misuse-resistance | Must not fail. |
| `response_to_bytes_and_back`          | Test misuse-resistance | Must not fail. |
| `authenticate_data_to_bytes_and_back` | Test misuse-resistance | Must not fail. |
| `idle_done_to_bytes_and_back`         | Test misuse-resistance | Must not fail. |

Three first five fuzz targets are used to test the parsing routines.
The fuzzers all do the same: try to parse the input from libFuzzer (and hope that the parsers don't crash), then,
if parsing was successful, serialize the obtained object (and hope that the serialization routines don't crash), and then,
parse the serialized output again and compare it to the first one (and hoping that they match).
This is motivated by the fact, that the library must certainly be able to parse the data it has produced on its own.

The last five fuzz targets are used to test for misuse-resistance.
The `Greeting`/`Command`/`Response`/... structs implements the `Arbitrary` trait that will produce a random instance of the type.
Any instance generated in this way must be parsable and valid.
It should not be possible to create a message object via the API, which is invalid according to the IMAP specification.

If a crash was found, it is helpful to use the `debug` feature and rerun the crashing input. 

## Try to be more effective

* Use `terminals.dict` as fuzzing dictionary. It contains all terminals (>1 character) from the IMAP4rev1 formal syntax and ABNFs core rules.
* The `imap.dict` dictionary contains a full IMAP trace. `blns.dict` is the "big list of naughty strings".
* Decrease the the input size to e.g. 64 bytes. Short inputs might still trigger complex parsing routines.
* Use multiple processes.
* Try to use `-ascii_only` to exclude inputs, which are less likely to be valid (useful to test serializing.)

```sh
cargo +nightly fuzz run <target> -j 32 -- -dict=terminals.dict -max_len=64 -only_ascii=1
```

## Structured fuzzing with `Arbitrary`

These beautiful commands ...

```rust
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, And([Answered, Deleted, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft]+)]+, uid: true } }
// z UID SEARCH UNDRAFT UNDRAFT (ANSWERED DELETED UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT)\r\n
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, And([Answered, Deleted, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft]+)]+, uid: true } }
------------------------------------------------------------------------------------------------------------------------
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Cc(String(Quoted(Quoted("")))), All]+, uid: false } }
// z SEARCH UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT CC \"\" ALL\r\n
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Cc(String(Quoted(Quoted("")))), All]+, uid: false } }
------------------------------------------------------------------------------------------------------------------------
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Cc(String(Literal(Literal { data: b"\x97\x97", mode: Sync })))]+, uid: false } }
// z SEARCH UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT CC {2}\r\n\x97\x97\r\n
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Undraft, Cc(String(Literal(Literal { data: b"\x97\x97", mode: Sync })))]+, uid: false } }
------------------------------------------------------------------------------------------------------------------------
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, Undraft, Undraft, Undraft, All, Undraft, Undraft, Undraft]+, uid: true } }
// z UID SEARCH UNDRAFT UNDRAFT UNDRAFT UNDRAFT UNDRAFT ALL UNDRAFT UNDRAFT UNDRAFT\r\n
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Undraft, Undraft, Undraft, Undraft, All, Undraft, Undraft, Undraft]+, uid: true } }
------------------------------------------------------------------------------------------------------------------------
Command { tag: Tag("-"), body: GetMetadata { options: [Depth(Infinity), Depth(Infinity)], mailbox: Other(MailboxOther(String(Quoted(Quoted(""))))), entries: [Entry(Atom(AtomExt("["))), Entry(String(Literal(Literal { data: b"[[", mode: NonSync }))), Entry(String(Quoted(Quoted("")))), Entry(String(Quoted(Quoted(""))))]+ } }
// - GETMETADATA (DEPTH INFINITY DEPTH INFINITY) \"\" ([ {2+}\r\n[[ \"\" \"\")\r\n
Command { tag: Tag("-"), body: GetMetadata { options: [Depth(Infinity), Depth(Infinity)], mailbox: Other(MailboxOther(String(Quoted(Quoted(""))))), entries: [Entry(Atom(AtomExt("["))), Entry(String(Literal(Literal { data: b"[[", mode: NonSync }))), Entry(String(Quoted(Quoted("")))), Entry(String(Quoted(Quoted(""))))]+ } }
------------------------------------------------------------------------------------------------------------------------
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Old, Old, Old, Old, Undraft, Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Literal(Literal { data: b"T\xf0&!\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd$-MxM\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd", mode: NonSync }))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("&")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Literal(Literal { data: b"", mode: Sync })))]+, uid: false } }
// z SEARCH UNDRAFT OLD OLD OLD OLD UNDRAFT BCC \"\" BCC \"\" BCC \"\" BCC {34+}\r\nT\xf0&!\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd$-MxM\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd BCC \"\" BCC \"&\" BCC \"\" BCC \"\" BCC {0}\r\n\r\n
Command { tag: Tag("z"), body: Search { charset: None, criteria: [Undraft, Old, Old, Old, Old, Undraft, Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Literal(Literal { data: b"T\xf0&!\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd$-MxM\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd\xbd", mode: NonSync }))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("&")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Quoted(Quoted("")))), Bcc(String(Literal(Literal { data: b"", mode: Sync })))]+, uid: false } }
------------------------------------------------------------------------------------------------------------------------
Command { tag: Tag("m="), body: Fetch { sequence_set: SequenceSet([Single(Asterisk)]+), macro_or_item_names: MessageDataItemNames([BodyExt { section: Some(HeaderFields(Some(Part([1547436031, 4294967117, 1308622847]+)), [Atom(AtomExt("iiiiiiiiiMM?"))]+)), partial: None, peek: false }, InternalDate, InternalDate, BodyExt { section: Some(Header(None)), partial: Some((1296845901, 4143333197)), peek: false }, BinarySize { section: [761019391, 1296895278] }, BodyExt { section: Some(HeaderFields(Some(Part([1533634921, 1768515945, 1768515945]+)), [Atom(AtomExt("!!?-MMM"))]+)), partial: Some((1296845901, 4143333197)), peek: false }, Flags, InternalDate, InternalDate, BinarySize { section: [1308622847, 4294967117, 1308622847] }, InternalDate, BinarySize { section: [] }, InternalDate, Envelope, BodyExt { section: Some(Header(None)), partial: Some((1296845901, 4143333197)), peek: false }, BinarySize { section: [761019391, 1296895278, 4143333197] }, BinarySize { section: [761019391, 1296895278] }, BodyExt { section: Some(HeaderFields(Some(Part([1768515945, 1298753897, 1280126382]+)), [String(Literal(Literal { data: b"U\xff\xff\xff;\\-M", mode: NonSync }))]+)), partial: None, peek: false }]), uid: false } }
// m= FETCH * (BODY[1547436031.4294967117.1308622847.HEADER.FIELDS (iiiiiiiiiMM?)] INTERNALDATE INTERNALDATE BODY[HEADER]<1296845901.4143333197> BINARY.SIZE[761019391.1296895278] BODY[1533634921.1768515945.1768515945.HEADER.FIELDS (!!?-MMM)]<1296845901.4143333197> FLAGS INTERNALDATE INTERNALDATE BINARY.SIZE[1308622847.4294967117.1308622847] INTERNALDATE BINARY.SIZE[] INTERNALDATE ENVELOPE BODY[HEADER]<1296845901.4143333197> BINARY.SIZE[761019391.1296895278.4143333197] BINARY.SIZE[761019391.1296895278] BODY[1768515945.1298753897.1280126382.HEADER.FIELDS ({8+}\r\nU\xff\xff\xff;\\-M)])\r\n
Command { tag: Tag("m="), body: Fetch { sequence_set: SequenceSet([Single(Asterisk)]+), macro_or_item_names: MessageDataItemNames([BodyExt { section: Some(HeaderFields(Some(Part([1547436031, 4294967117, 1308622847]+)), [Atom(AtomExt("iiiiiiiiiMM?"))]+)), partial: None, peek: false }, InternalDate, InternalDate, BodyExt { section: Some(Header(None)), partial: Some((1296845901, 4143333197)), peek: false }, BinarySize { section: [761019391, 1296895278] }, BodyExt { section: Some(HeaderFields(Some(Part([1533634921, 1768515945, 1768515945]+)), [Atom(AtomExt("!!?-MMM"))]+)), partial: Some((1296845901, 4143333197)), peek: false }, Flags, InternalDate, InternalDate, BinarySize { section: [1308622847, 4294967117, 1308622847] }, InternalDate, BinarySize { section: [] }, InternalDate, Envelope, BodyExt { section: Some(Header(None)), partial: Some((1296845901, 4143333197)), peek: false }, BinarySize { section: [761019391, 1296895278, 4143333197] }, BinarySize { section: [761019391, 1296895278] }, BodyExt { section: Some(HeaderFields(Some(Part([1768515945, 1298753897, 1280126382]+)), [String(Literal(Literal { data: b"U\xff\xff\xff;\\-M", mode: NonSync }))]+)), partial: None, peek: false }]), uid: false } }
```

... were generated by ...

```sh
cargo +nightly fuzz run --features=ext,debug command_to_bytes_and_back --release
```

Happy fuzzing!

# Known crashes

None of the targets should crash anymore.
However, they already uncovered interesting serialization issues.
Please try for yourself and file a bug report if you can do it!

# Fuzzing IMAP extensions

You can use the `ext` feature to activate most IMAP extensions.
Note, however, that some extensions are still experimental and may crash.
If so, please file a bug with the crashing input (and enabled `debug` feature).
