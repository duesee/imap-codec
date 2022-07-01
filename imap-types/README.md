# Misuse-resistant IMAP Types

This library provides types, i.e., `struct`s and `enum`s, to support [IMAP4rev1] implementations.
The types were initially extracted from [imap-codec] -- an IMAP parser and serializer -- and may now serve as a common basis for diverse IMAP implementations in Rust.

## Features

* Rust's type system is used to enforce correctness and make the library misuse-resistant. 
It must not be possible to construct a type that violates the IMAP specification.
* Fuzzing (via [cargo fuzz]) and property-based tests are used to uncover bugs.
The library is fuzz-tested never to produce an invalid message.

## Core Types

To ensure correctness, imap-types makes use of types such as
[AString](core::AString),
[Atom](core::Atom),
[IString](core::IString),
[Quoted](core::Quoted), and
[Literal](core::Literal) (from the [core] module).
It is good to know these types, because IMAP may require different message flows depending on what type was used to encode certain information such as a username or password.
When constructing types, imap-types will automatically choose the best encoding.

# License

This crate is dual-licensed under Apache 2.0 and MIT terms.

[IMAP4rev1]: https://datatracker.ietf.org/doc/html/rfc3501
[imap-codec]: https://github.com/duesee/imap-codec
[cargo fuzz]: https://github.com/rust-fuzz/cargo-fuzz
[core]: https://docs.rs/imap-types/latest/imap_types/core/index.html
