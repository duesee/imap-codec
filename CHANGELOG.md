# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - YYYY-MM-DD

## [Version 0.7.0] - 2022-08-01

### Added

* Add tokio demos (client + server).
* Introduce `ImapClientCodec` and implement `tokio_util::codec::{Encoder, Decoder}`.
* Add tests to `tokio_compat`.
* Add `greeting_to_bytes_and_back` fuzz target.
* Introduce `Decode` trait and implement it for `Command` and `Response`.
* Introduce `Greeting`, and `GreetingKind`.
* Introduce `State::Greeting` variant.
* Introduce `IdleDone`.
* Introduce `CapabilityOther` and implement `Capability::other()`.
* Implement `Decode` for `Greeting` and use it in the `tokio_compat` module.
* Implement `AuthMechanism::other`.
* Implement `Data::expunge`.
* Implement `Code::{uidnext, uidvalidity, unseen}`.

### Changed

* Improve documentation.
* Switch to new module layout in imap-codec.
* Refactor creation of `Command`s and `CommandBody`s.
* Use `Decode` trait in examples.
* Use `Command::decode` instead of `command`.
* Allow "Unicode-DFS-2016" license in "deny.toml".
* Use `Tag` in `State::{IdleAuthenticated,IdleSelected}` instead of `String`.
* Derive `Debug`, `Eq`, and `PartialEq` for `State`.
* Feature-gate `Capability::LoginDisabled` with "starttls" feature.
* Feature-gate `State::{Idle*}` variants with "ext_idle" feature.

### Removed

* Remove `nom` feature.
* Don't export `arbitrary`, and `rfc3501`.
* Make `imap_types::{codec, state}` part of public API. Don't export `imap_types::Encode` directly.
* Delete `greeting` constructor of `Status`.
* Delete `PreAuth` variant (and constructor) of `Status`.

### Fixed

* Fix (and improve) examples.

## [Version 0.6.0] - 2022-06-14

### Added

- Introduce "starttls" feature.
  - Cleanup and document existing features.
- Measure code coverage in CI.
  - Upload coverage report to Coveralls.io.
  - Add/Update code coverage badge in README.md.
- Compile fuzzers in CI.
- Implement benchmarks (Criterion.rs).
  - Compile benchmarks in CI.
- Implement `Command::into_static()` and `Response::into_static()`.
  - Use `bounded-static` (thanks, @jakoschiko)
- Add types to fix misuses
  - Introduce `AtomExt` (1*ASTRING-CHAR) to fix misuse.
  - Introduce `CapabilityEnable` to increase misuse-resistance.
- Split imap-codec into imap-codec and imap-types.
  - Implement non-nom parsing in imap-types.
  - Add README.md to imap-types.

### Changed

- Split crate into imap-codec and imap-types.
  - Make imap-codec the primary workspace member.
  - Re-export `imap-types`.
- Make fuzz targets members of workspace to simplify workflow.
- Rename "serdex"/"nomx" features to "serde"/"nom".
- Reduce allocations during parsing.
  - Use `Cow` to abstract over owned and borrowed slices.
- Do not check slices twice.
  - Introduce `new_unchecked()` functions.
  - Check `new_unchecked()` during debug builds.
- Cleanup API for `AuthMechanism`.
- Update to nom 7 and abnf-core 0.5.

### Removed

- Remove `impl Display` for types in imap-types.
- Remove `nom` feature in imap-types.
- Remove/cleanup (unused) dependencies in imap-codec.
- Remove/cleanup (unused) dependencies in imap-types.

### Fixed

- Fix fuzz targets.
- Fix benchmarks (thanks, @franziskuskiefer).
- Fix misuses, e.g., `AtomExt` (1*ASTRING-CHAR).

[Unreleased]:    https://github.com/duesee/imap-codec/compare/6df4364620e0d64566e4291797a3c9955af422ea...HEAD
[Version 0.7.0]: https://github.com/duesee/imap-codec/compare/63b6a2e4a94f2734d67a18039b3f6dae68994902...6df4364620e0d64566e4291797a3c9955af422ea
[Version 0.6.0]: https://github.com/duesee/imap-codec/compare/fcb400e508f74a8d88bbcbfd777bdca7cb75bdeb...63b6a2e4a94f2734d67a18039b3f6dae68994902
