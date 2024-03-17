# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - YYYY-MM-DD

### Added

* Implemented more IMAP extensions
  * ID
  * UNSELECT
  * SORT and THREAD
  * BINARY
  * METADATA
* Implemented `AuthenticateData::Cancel`
* Implemented `AuthMechanism::ScramSha3_512{,Plus}`
* Implemented more common traits for types
  * Thanks, @jakoschiko!
* Implemented missing tests
* Added `arbitrary_simplified` feature
* Added `Vec2`
* Added short `README.md` to `assets` folder
* Added quirk for trailing space in STATUS. Thanks, @nbdd0121!

### Changed

* Changed `Status` to make it easier to use
* Check only explicit features for SemVer violations
* Renamed `NonEmptyVec` to `Vec1`
* Updated `CONTRIBUTING.md`

### Fixed

* Fixed examples in README (and test them in CI now)
  * Thanks, @coalooball!
* Fixed broken links in README
* Fixed iteration over sequence numbers
  * Thanks, @superboum!
* Don't log `Rectified missing text to \"...\"` unnecessarily
* Made `{DateTime,NaiveDate}::unvalidated` `panic!` in debug on wrong input
* Mention `panic!` in `unvalidated` documentation
* Fixed typo in `AuthMechanism` documentation

## [Version 1.0.0] - 2023-08-22

### Changed

* Use `'static` lifetime for `Decoder::Error` in `decode_static`.

## [Version 1.0.0-beta] - 2023-08-17

### Added

* Introduced `FlagNameAttributeExtension`.
* Implemented `Display` for some `T` where `T`s `Display` implementation equals `Encode`.

### Changed

* Inlined stable `ext_*` features to improve SemVer compatibility.
* Re-exported `imap_codec::imap_types`.
* Simplified module hierarchy.
* Increased MSRV to 1.65.
* Moved tokio implementation to demos.
* Replaced `Decode` trait with `Decoder`.
* Replaced `Encode` trait with `Encoder`.
* Made `*Other` types merely technicalities.
* Improved `Debug` print.
* Updated `Swatinem/rust-cache`.
* Simplified `Error`s.
* Aligned type names with IMAP RFC.
* Improved documentation.
* Replaced `Capability::Literal(LiteralCapability)` with `Capability::LiteralPlus`, and `Capability::LiteralMinus`.

## [Version 0.10.0] - 2023-07-05

### Added

* Added `AuthMechanism::XOAUTH`.
* Added more constructors.
* Added (and improved) feature documentation. (Thanks, @jakoschiko!)
* Added multiple `quirk_*` features to improve interoperability.
* Added `DecodeStatic`.
* Checking with `cargo-hack` and `--feature-powerset`.
* Fuzz-testing with incomplete messages.

### Changed

* Simplified module hierarchy.
* Renamed types for better understandability (and to align them with the IMAP4rev1 standard).
* Renamed constructors so they cannot be confused with `unsafe`. (Thanks, @jakoschiko!)
* Resolved multiple SemVer hazards.
* Use custom nom error.
* Deduplicated (and added a new) fuzz-target(s).
* Don't export nom parsers anymore.
* Removed constant-time comparison support.
* Simplified `Debug`ing of `NonEmptyVec`.

### Fixed

* Fixed warnings and broken links in documentation.
* Fixed `is_text_char`.
* Fixed `condstore` identity.
* Fixed usage of `complete` (instead of `streaming`).

### Removed

* Removed `ansi_term` dev dependency.

## [Version 0.9.0] - 2023-05-30

### Added

- Implemented `MOVE` (RFC 6851).
- Implemented `UNSELECT` (RFC 3691).
- Implemented (some of) `CONDSTORE` / `QRESYNC` (RFC 7162).
- Reworked (and enabled) coverage job in CI.
- Added (spot-)fuzzing to CI.
- Added `minimal-versions` job to CI.
  - Test MSRV.
  - Test lowest versions of dependencies.

### Changed

- Migrated to Rust 2021.
- Redesigned `Encode` trait.
- Moved `Encode` trait from imap-types to imap-codec.

### Fixed

- Made known-answer tests stronger.
  - Made it so that `Decode` is always tested during `Encode` and vice versa.
- Made it so that random tests are reproducable through a seed.
- Resolved remaining `TODO`s in `command_to_bytes_and_back` fuzz-target.
- Resolved remaining `TODO`s in `{Single,Multi}PartExtensionData`
  - Fixed misuse of `{Single,Multi}PartExtensionData.`
  - Introduced `BodyExtension`.
- Introduced `ContinueBasic` to prevent ambiguities.
- Fixed `Eq` side effect of `Secret`.
- Fixed `mbx_list_flags`.
- Fixed `NaiveDate`.
  - Made `MyNaiveDate::arbitrary` really arbitrary.
  - Narrowed allowed values for `DateTime` and `NaiveDate`.
- Fixed poor constant-time sanity check.
- Fixed possible `panic!` in `response`.
- Reactivated ignored tests.

## [Version 0.8.0] - 2023-04-16

### Added

* Community
  * Introduced a project board and a GitHub action that adds all opened issues to the project board.
  * Added a `CONTRIBUTING.md`.
* Features
  * Implemented RFC 2088/RFC 7888 (LITERAL+).
  * Implemented RFC 2087/RFC 9208 (QUOTA).
    * Thanks, @MinisculeGirraffe!
  * Introduced usable error reporting.
  * Introduced `Encode::encode_detached`.
  * Implemented missing `From`, `TryFrom`, `AsRef`, ... conversions for various types.
* Testing/Fuzzing
  * Improved debug workflow.
  * Introduced `ext` and `debug` features.
* Security
  * Forbid `unsafe` and introduced `unchecked` feature.
  * Ensured that secret values are not `Debug`-printed and comparisons are made in constant time.
    * Wrapped `AuthenticateData` in `Secret`.
    * Wrapped `CommandBody::Login.password` in `Secret`.

### Changed

* Refactoring
  * Feature-gated all existing extensions.
  * Simplified module/feature names for `tokio` support.
  * Changed naming schema to phase out `mod.rs`.
  * Renamed `MyDateTime` to `DateTime`, `SeqNo` to `SeqOrUid`, `SeqNo::Largest` to `SeqNo::Asterisk`.
* CI
  * Added a job that checks for SemVer violations.
  * Improved CI runtime.
    * Made it so that superseded jobs are eagerly canceled.
    * Made it so that the `Coverage` job is started only after a successful `Build & Test`.
    * Inlined `--all-features` to reduce compilation time.
* Chore
  * Allowed `Unicode-DFS-2016` and `BSD-3-Clause` dependencies.

### Fixed

* Testing/Fuzzing
  * Made fuzz-targets tighter by not skipping (known) misuses.
  * Reactivated commented-out test code.
  * Restored trace generation for `README.md`.
* Misuses
  * Fixed (known) misuses for `Capability{,Other}`, `Code{,Other}`, `Continue`, `Flag`, and `Body`.
    * Worked around ambiguities in IMAP.
    * Fixed various parsers that need to greedily consume tokens such as `Atom`s.
  * Fixed `text` parser by excluding `[` and `]`.

## [Version 0.7.0] - 2022-08-05

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

* Improve CI.
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

* Fix missing doc test in CI.
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

[Version 0.6.0]:      https://github.com/duesee/imap-codec/compare/fcb400e508f74a8d88bbcbfd777bdca7cb75bdeb...63b6a2e4a94f2734d67a18039b3f6dae68994902
[Version 0.7.0]:      https://github.com/duesee/imap-codec/compare/63b6a2e4a94f2734d67a18039b3f6dae68994902...16e34bce239840bc3a39c811f1ce3d36c6ea20b0
[Version 0.8.0]:      https://github.com/duesee/imap-codec/compare/16e34bce239840bc3a39c811f1ce3d36c6ea20b0...f5138ac09b6e160256c8e6dc80db1597aee92394
[Version 0.9.0]:      https://github.com/duesee/imap-codec/compare/f5138ac09b6e160256c8e6dc80db1597aee92394...3bb1b380a6f163a16732f9dd9c8382f2af73868c
[Version 0.10.0]:     https://github.com/duesee/imap-codec/compare/3bb1b380a6f163a16732f9dd9c8382f2af73868c...ca3ef319681d4e8ea2daf28b9a3650d2d74813c7
[Version 1.0.0-beta]: https://github.com/duesee/imap-codec/compare/ca3ef319681d4e8ea2daf28b9a3650d2d74813c7...1b8924dce7c943cd003a8316f384af97649feadf
[Version 1.0.0]:      https://github.com/duesee/imap-codec/compare/1b8924dce7c943cd003a8316f384af97649feadf...a5d8dff9e8047bda2c477a3a9d56e53274113b26
[Unreleased]:         https://github.com/duesee/imap-codec/compare/a5d8dff9e8047bda2c477a3a9d56e53274113b26...HEAD
