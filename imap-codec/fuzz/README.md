# Fuzzing

## Setup

Cargo fuzz requires a nightly compiler. You can install it via ...

```sh
rustup install nightly
```

... and invoke it by adding the "+nightly" flag to cargo like ...

```sh
cargo +nightly fuzz <target>
```

Alternatively, you can override the default toolchain for the current directory by using ...

```sh
rustup override set nightly
```

Don't forget to unset it with ...

```sh
rustup override unset
```

... as imap-codec should work with stable.

## Provided fuzz targets

You can start the fuzzing process by running ...

```sh
cargo +nightly fuzz run <target>
```

... with `<target>` being ...

| Name of the fuzz target `<target>` | Purpose                | Expectation    |
|------------------------------------|------------------------|----------------|
| `greeting`                         | Test parsing           | Must not fail. |
| `command`                          | Test parsing           | Must not fail. |
| `response`                         | Test parsing           | Must not fail. |
| `greeting_to_bytes_and_back`       | Test misuse-resistance | Must not fail. |
| `command_to_bytes_and_back`        | Test misuse-resistance | Must not fail. |
| `response_to_bytes_and_back`       | Test misuse-resistance | Must not fail. |

Three first three fuzz targets are used to test the parsing routines.
The fuzzers all do the same: try to parse the input from libFuzzer (and hope that the parsers don't crash), then,
if parsing was successful, serialize the obtained object (and hope that the serialization routines don't crash), and then,
parse the serialized output again and compare it to the first one (and hoping that they match).
This is motivated by the fact, that the library must certainly be able to parse the data it has produced on its own.

The last three fuzz targets are used to test for misuse-resistance and currently are a work-in-progress.
The `Greeting`/`Command`/`Response`/... structs implements the `Arbitrary` trait that will produce a random instance of the type.
Any instance generated in this way must be parsable and valid.
It should not be possible to create a message object via the API, which is invalid according to the IMAP specification.

If a crash was found, it is helpful to uncomment the `println!(...)` statements in the fuzz target and rerun the crashing input. 

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

This beautiful `Command`¹ ...

```rust
Command {
    tag: Tag(
        "!",
    ),
    body: Fetch {
        sequence_set: SequenceSet(
            [
                Single(
                    Value(
                        7,
                    ),
                ),
            ],
        ),
        attributes: FetchAttributes(
            [
                BodyExt {
                    section: Some(
                        HeaderFieldsNot(
                            Some(
                                Part(
                                    NonEmptyVec(
                                        [
                                            1768386412,
                                        ],
                                    ),
                                ),
                            ),
                            NonEmptyVec(
                                [
                                    String(
                                        Quoted(
                                            Quoted(
                                                "",
                                            ),
                                        ),
                                    ),
                                    Atom(
                                        Atom(
                                            "`",
                                        ),
                                    ),
                                ],
                            ),
                        ),
                    ),
                    partial: None,
                    peek: false,
                },
                BodyExt {
                    section: Some(
                        HeaderFieldsNot(
                            None,
                            NonEmptyVec(
                                [
                                    String(
                                        Quoted(
                                            Quoted(
                                                "",
                                            ),
                                        ),
                                    ),
                                    Atom(
                                        Atom(
                                            "!`",
                                        ),
                                    ),
                                ],
                            ),
                        ),
                    ),
                    partial: None,
                    peek: false,
                },
                BodyExt {
                    section: Some(
                        HeaderFieldsNot(
                            None,
                            NonEmptyVec(
                                [
                                    String(
                                        Quoted(
                                            Quoted(
                                                "",
                                            ),
                                        ),
                                    ),
                                    String(
                                        Literal(
                                            Literal(
                                                [],
                                            ),
                                        ),
                                    ),
                                ],
                            ),
                        ),
                    ),
                    partial: None,
                    peek: false,
                },
            ],
        ),
        uid: false,
    },
}
```

... was generated by `Command::arbitrary(...)` and serializes into ...


```imap
! FETCH 7 (BODY[1768386412.HEADER.FIELDS.NOT ("" `)] BODY[HEADER.FIELDS.NOT ("" !`)] BODY[HEADER.FIELDS.NOT ("" {0}
)])
```

# Known crashes

I am not able to crash the `greeting`, `command`, and `response` targets anymore.
However, they already uncovered interesting serialization issues.
Similarly, I can not create any invalid `Greeting` or `Command` anymore.
Please try for yourself and file a bug report if you can do it!

¹ This may become outdated when new versions are published.