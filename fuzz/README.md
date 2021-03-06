# Fuzzing

## Use Nightly

Cargo fuzz requires nightly. You can override the toolchain by using

```sh
rustup override set nightly
```

Don't forget to unset it with

```sh
rustup override unset
```

as imap-codec should work with stable.

## Run the fuzzing process

Three fuzzing targets are defined `greeting`, `command`, and `response`. They can be run with

```sh
cargo fuzz run <target>
```

The fuzzers all do the same: try to parse the input from libFuzzer (and hope that the parsers don't crash), then, if parsing was successful, serialize the obtained object (and hope that the serialization routines don't crash), and then, parse the serialized output again and compare it to the first one.

This is motivated by the fact, that the library must certainly be able to parse the data it has produced on its own.

If a crash was found, it is helpful to uncomment the `println` statements in the fuzzing target and rerun the crashing input. 

# Making fuzzing more effective

* Use `terminals.dict` as fuzzing dictionary. It contains all terminals (>1 character) from the IMAP4rev1 formal syntax and ABNFs core rules.
* The `imap.dict` dictionary contains a full IMAP trace. `blns.dict` is the "big list of naughty strings".
* Decrease the the input size to e.g. 64 bytes. Short inputs might still trigger complex parsing routines.
* Use multiple processes.
* Try to use `-ascii_only` to exclude inputs, which are less likely to be valid (useful to test serializing.)

```sh
cargo fuzz run <target> -j 32 -- -dict=terminals.dict -max_len=64 -only_ascii=1
```

# Structured fuzzing with `Arbitrary`

TBD :-)

# Known crashes

I am not able to crash the `greeting`, `command`, and `response` targets anymore. However, they already uncovered interesting serialization issues.
