# Welcome to imap-codec's (and imap-types') contributing guide

Thanks for investing your time to help with this project! Keep in mind that this project is driven by volunteers. Be patient and polite, and empower others to improve. Always use your best judgment and be excellent to each other.

## Principles

### Misuse resistance

We use strong-typing to [eliminate invalid state].
Ask yourself: Can I instantiate a type with an invalid variable setting?
If yes, consider how to eliminate it.
If you're unsure, let's figure it out together!

## Project management

We use the [just](https://github.com/casey/just) command runner for Continuous Integration (CI).
The GitHub Actions infrastructure merely calls `just` to execute jobs.
This means that you can run all required tests for a PR using `just ci`.

### Code formatting

Code is formatted using [`rustfmt`] with a custom `rustfmt.toml` and checked in CI.
The config minimizes diffs and ensures a consistent style of imports.
Sadly, some config options (still) require a nightly compiler.
Thus, you need to use `cargo +nightly fmt`.

Note: Code formatting and fuzzing should be the only routines requiring a nightly compiler.
Everything else must work on stable!

### SemVer violations

Breaking API changes (w/o a corresponding version bump) are detected in CI using the [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks).

### MSRV

The Minimum Supported Rust Version (MSRV) is 1.65 and [checked in CI](https://github.com/duesee/imap-codec/blob/main/.github/workflows/build_and_test.yml#L116C36-L116C40).

### IMAP extensions

> [!WARNING]
> This is a best effort with various shortcomings. Please tell us if you have a better solution!

IMAP is extensible.
Thus, we use [Cargo features] to enable/disable extensions to the core IMAP protocol.
Feature-gating helps to reduce the amount of exposed code and serves as documentation for the supported extensions.

The current idea is: Every extension starts as a feature-gated "experimental" extension.
This way, we can exclude it from SemVer checks.
When we think an extension is "right", we can remove the feature gate.
Note, however, that -- depending on the feature -- a major version bump might still be required to get it in.
Thus, let's verify what features are ready close to major releases.

## Testing

There are multiple forms of testing in `imap-codec`. 

### Known-answer tests

Known-answer tests are used to ensure that a specific IMAP message is *really* parsed into an expected object.
We usually extract examples from a specific RFC and encode our expectations as unit tests.
To implement this test, you can use `kat_inverse_{greeting,command,response,...}`.

### Fuzzing

Fuzzing is used in `imap-types` and `imap-codec`.
Fuzzing is used to test that parsing and serialization are inverses of each other (which already helped uncover a lot of bugs).
For more information, see [imap-codec/fuzz/README.md](imap-codec/fuzz/README.md).
The CI runs a limited number (25.000) of fuzz runs.

### Regressions & fixed bugs

When fixing a bug, we should add a test to 1) show how to reproduce the bug and 2) show that a fix is effective.
This also ensures that we refrain from reintroducing this bug in the future, e.g., during an incorrect refactoring.

### API tests & doc tests

When explaining how the API should be used, we should write the example as a test.
This way, we ensure that examples stay relevant and that we don't accidentally change the API.

[`rustfmt`]: https://github.com/rust-lang/rustfmt
[eliminate invalid state]: https://duesee.dev/p/type-driven-development/
[Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
