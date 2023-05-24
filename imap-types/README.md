# Misuse-resistant IMAP Types

This crate provides common types, i.e., `struct`s and `enum`s, to support [IMAP] implementations.
It tries to become the "standard library" for IMAP in Rust that can be used as a common basis for a diverse set of IMAP
crates, such as parsers, serializers, clients, and servers.
If you are looking for a rock-solid IMAP "codec" implementation, i.e., parsers and serializers, that uses imap-types,
see [imap-codec].

## Features

* Rust's type system is used to enforce correctness and to make the library misuse-resistant.
  It's not possible to construct a message that violates the IMAP specification.
* Fuzzing (via [cargo fuzz]) and property-based tests are used to uncover bugs.
  The library is fuzz-tested never to produce an invalid message.

## Working with imap-types

To ensure correctness, imap-types makes use of types such as
[AString](core::AString),
[Atom](core::Atom),
[IString](core::IString),
[Quoted](core::Quoted), and
[Literal](core::Literal) (from the [core] module).
It's good to know these types because IMAP requires different message flows depending on how an information, such as a
username or password is represented, e.g., as an atom or literal.
When constructing messages, imap-types can automatically choose the best representation.
However, it's always possible to manually choose a specific representation.

### Examples

<details>
<summary>Automatic Construction</summary>

This ...

```rust
Command::new(
    "A1",
    CommandBody::login("alice", "password").unwrap(),
).unwrap();
```

... will produce ...

```imap
A1 LOGIN alice password
```

However, ...

```rust
Command::new(
    "A1",
    CommandBody::login("alice\"", b"\xCA\xFE".as_ref()).unwrap(),
)
.unwrap();
```

... will produce ...

```imap
A1 LOGIN "alice\"" {2}
\xCA\xFE
```

Also, the construction ...

```rust
Command::new(
    "A1",
    CommandBody::login("alice\x00", "password").unwrap(),
).unwrap();
```

... will fail because IMAP doesn't allow NULL bytes in the username (nor password).
</details>

<details>
<summary>Manual Construction</summary>

You can also use ...

```rust
Command::new(
    "A1",
    CommandBody::login(Literal::try_from("alice").unwrap(), "password").unwrap(),
)
.unwrap();
```

... to produce ...

```imap
A1 LOGIN {5}
alice password
```

... even though "alice" could be encoded more simply with an atom or quoted string.

Also, you can use Rust literals and resort to `unchecked` constructors when you are certain that your input is correct:

```rust
// This could be provided by the email application.
let tag = TagGenerator::random();

Command {
    tag,
    body: CommandBody::Login {
        // Note that the "unchecked" feature must be activated.
        username: AString::from(Atom::unchecked("alice")),
        password: Secret::new(AString::from(Atom::unchecked("password"))),
    },
};
```

In this case, imap-codec won't stand in your way.
However, it won't guarantee that you produce correct messages, either.
</details>

# License

This crate is dual-licensed under Apache 2.0 and MIT terms.

[IMAP]: https://datatracker.ietf.org/doc/html/rfc3501
[imap-codec]: https://github.com/duesee/imap-codec
[cargo fuzz]: https://github.com/rust-fuzz/cargo-fuzz
[core]: https://docs.rs/imap-types/latest/imap_types/core/index.html
