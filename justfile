export RUSTFLAGS := "-D warnings"
export RUSTDOCFLAGS := "-D warnings"

[private]
default:
    just -l --unsorted

# Install Rust 1.65, Rust nightly, cargo-deny, cargo-fuzz, cargo-hack, and cargo-semver-checks
install: install_rust_1_65 \
         install_rust_nightly \
         install_cargo_clippy \
         install_cargo_deny \
         install_cargo_fuzz \
         install_cargo_hack \
         install_cargo_semver_checks

# Required to check MSRV
[private]
install_rust_1_65:
    rustup toolchain install 1.65 --profile minimal

# Required for code formatting and fuzzing
[private]
install_rust_nightly:
    rustup toolchain install nightly

[private]
install_cargo_clippy:
    rustup component add clippy

[private]
install_cargo_deny:
    cargo install --locked cargo-deny
 
[private]
install_cargo_fuzz: install_rust_nightly
    cargo install cargo-fuzz

[private]
install_cargo_hack:
    cargo install --locked cargo-hack

[private]
install_cargo_semver_checks:
    cargo install --locked cargo-semver-checks

# Check syntax, formatting, clippy, deny, ...
quick: (quick_impl ""               ""         ) \
       (quick_impl ""               "--release") \
       (quick_impl "--all-features" ""         ) \
       (quick_impl "--all-features" "--release")

[private]
quick_impl features mode: (cargo_check features mode) cargo_fmt (cargo_clippy features mode) cargo_deny

[private]
cargo_check features mode:
    cargo check --workspace --all-targets {{features}} {{mode}}

[private]
cargo_fmt: install_rust_nightly
    cargo +nightly fmt --check

[private]
cargo_clippy features mode: install_cargo_clippy
    cargo clippy --workspace --all-targets {{features}} {{mode}}

[private]
cargo_deny: install_cargo_deny
    cargo deny check

# Check SemVer breaking changes
semver: install_cargo_semver_checks
    cargo semver-checks check-release --only-explicit-features -p imap-codec
    cargo semver-checks check-release --only-explicit-features -p imap-types

# Check extensively (required for PR)
pr: (pr_impl ""               ""         ) \
    (pr_impl ""               "--release") \
    (pr_impl "--all-features" ""         ) \
    (pr_impl "--all-features" "--release")

[private]
pr_impl features mode: quick semver (cargo_hack mode) (cargo_test features mode)

[private]
cargo_hack mode: install_cargo_hack
    cargo hack check {{mode}}
    cargo hack check -p imap-codec \
        --no-dev-deps \
        --exclude-features default \
        --feature-powerset \
        --group-features \
        starttls,\
        ext_condstore_qresync,\
        ext_login_referrals,\
        ext_mailbox_referrals,\
        ext_id,\
        ext_sort_thread,\
        ext_binary,\
        ext_metadata \
        --group-features \
        quirk_crlf_relaxed,\
        quirk_rectify_numbers,\
        quirk_missing_text,\
        quirk_id_empty_to_nil,\
        quirk_trailing_space \
        {{mode}}
    cargo hack check -p imap-types \
        --no-dev-deps \
        --feature-powerset \
        --group-features \
        arbitrary,\
        arbitrary_simplified,\
        bounded-static,\
        serde \
        --group-features \
        starttls,\
        ext_condstore_qresync,\
        ext_login_referrals,\
        ext_mailbox_referrals,\
        ext_id,\
        ext_sort_thread,\
        ext_binary,\
        ext_metadata \
        {{mode}}

[private]
cargo_test features mode:
    cargo test \
    --workspace \
    --all-targets \
    --exclude imap-types-fuzz \
    --exclude imap-codec-fuzz \
    {{features}} \
    {{mode}}

# Benchmark
bench: cargo_bench

[private]
cargo_bench:
    cargo bench -p imap-codec --all-features

# Fuzz
[linux]
fuzz runs="25000": install_cargo_fuzz
    #!/usr/bin/env bash
    set -euo pipefail
    cd imap-codec
    for fuzz_target in $(cargo +nightly fuzz list)
    do
        echo "# Fuzzing ${fuzz_target}";
        cargo +nightly fuzz run --features=ext,arbitrary_simplified ${fuzz_target} -- -dict=fuzz/terminals.dict -max_len=256 -only_ascii=1 -runs={{runs}};
    done
