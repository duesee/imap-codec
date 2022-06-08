# Misuse-resistant Types for the IMAP Protocol

This library provides types, i.e., `struct`s and `enum`s, for the [IMAP4rev1] protocol.
Rust's type system is used to enforce correctness and make the library misuse-resistant.
It must not be possible to construct messages that violate the IMAP specification.

# Testing

Fuzzing (via [cargo fuzz]) and property-based tests are used to uncover bugs.
For example, the library is fuzz-tested never to produce an invalid message.
Testing is currently done in [imap-codec] -- the parent library from which imap-types was initially extracted.

# Used String Types

Due to the correctness guarantees, imap-types uses multiple ["string types"] like `Atom`, `Tag`, `NString`, and `IString`.
These types should not play an essential role for library consumers.
However, it can make sense to expose some of them.

# License

This crate is dual-licensed under Apache 2.0 and MIT terms.

[IMAP4rev1]: https://datatracker.ietf.org/doc/html/rfc3501
[cargo fuzz]: https://github.com/rust-fuzz/cargo-fuzz
[imap-codec]: https://github.com/duesee/imap-codec
["string types"]: https://docs.rs/imap-types/0.5.0/imap_types/core/index.html
