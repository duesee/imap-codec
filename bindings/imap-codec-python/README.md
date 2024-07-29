# imap-codec

This library provides parsing and serialization for [IMAP4rev1].
It is based on [`imap-codec`], a building block for IMAP client and server implementations written
in Rust, which implements the complete [formal syntax] of IMAP4rev1 and several IMAP extensions.

## Usage

```python
from imap_codec import Greeting, GreetingCodec

buffer = b"* OK Hello, World!\r\n<remaining>"

# Decode buffer into a greeting
remaining, greeting = GreetingCodec.decode(buffer)
assert remaining == b"<remaining>"
assert isinstance(greeting, Greeting)

# Extract greeting data as dictionary
data = greeting.as_dict()
assert data["code"] is None
assert data["kind"] == "Ok"
assert data["text"] == "Hello, World!"
```

For more usage examples take a look at the [examples] and [tests] on GitHub.

> **Note**: Access to data of message types (e.g. `Greeting`) is currently only available through
> dictionary representations (as seen above). This is planned to be improved in future releases of
> this library.

## License

This library is dual-licensed under Apache 2.0 and MIT terms.

[IMAP4rev1]: https://tools.ietf.org/html/rfc3501
[`imap-codec`]: https://crates.io/crates/imap-codec
[formal syntax]: https://tools.ietf.org/html/rfc3501#section-9
[examples]: https://github.com/duesee/imap-codec/tree/main/bindings/imap-codec-python/examples
[tests]: https://github.com/duesee/imap-codec/tree/main/bindings/imap-codec-python/tests
