export RUSTFLAGS := "-D warnings"
export RUSTDOCFLAGS := "-D warnings"

msrv := `sed -rn 's|^rust-version = \"(.*)\"$|\1|p' Cargo.toml`

[private]
default:
    just -l --unsorted

###########
### RUN ###
###########

# Run (local) CI
ci: (ci_impl ""           ""               ) \
    (ci_impl ""           " --all-features") \
    (ci_impl " --release" ""               ) \
    (ci_impl " --release" " --all-features")

[private]
ci_impl mode features: (check_impl mode features) (test_impl mode features)

# Check syntax, formatting, clippy, deny, semver, ...
check: (check_impl ""           ""               ) \
       (check_impl ""           " --all-features") \
       (check_impl " --release" ""               ) \
       (check_impl " --release" " --all-features")

[private]
check_impl mode features: (cargo_check mode features) \
                          (cargo_hack mode) \
                          cargo_fmt \
                          (cargo_clippy mode features) \
                          cargo_deny \
                          cargo_semver

[private]
cargo_check mode features:
    cargo check --workspace --all-targets --exclude imap-codec-bench{{ mode }}{{ features }}
    cargo doc --no-deps --document-private-items --keep-going{{ mode }}{{ features }}

[private]
cargo_hack mode: install_cargo_hack
    cargo hack check --workspace --all-targets --exclude imap-codec-bench{{ mode }}
    cargo hack check -p imap-codec \
        --no-dev-deps \
        --exclude-features default \
        --feature-powerset \
        --group-features \
        arbitrary,\
        arbitrary_simplified,\
        serde,\
        tag_generator \
        --group-features \
        starttls,\
        ext_condstore_qresync,\
        ext_id,\
        ext_login_referrals,\
        ext_mailbox_referrals,\
        ext_metadata,\
        ext_namespace,\
        ext_utf8 \
        --group-features \
        quirk_crlf_relaxed,\
        quirk_id_empty_to_nil,\
        quirk_missing_text,\
        quirk_rectify_numbers,\
        quirk_excessive_space_quota_resource,\
        quirk_trailing_space_capability,\
        quirk_trailing_space_id,\
        quirk_trailing_space_search,\
        quirk_trailing_space_status,\
        quirk_spaces_between_addresses,\
        quirk_empty_continue_req,\
        quirk_body_fld_enc_nil_to_empty\
        {{ mode }}
    cargo hack check -p imap-types \
        --no-dev-deps \
        --feature-powerset \
        --group-features \
        arbitrary,\
        arbitrary_simplified,\
        serde,\
        tag_generator \
        --group-features \
        starttls,\
        ext_condstore_qresync,\
        ext_id,\
        ext_login_referrals,\
        ext_mailbox_referrals,\
        ext_metadata,\
        ext_namespace,\
        ext_utf8\
        {{ mode }}
	
[private]
cargo_fmt: install_rust_nightly install_rust_nightly_fmt
    cargo +nightly fmt --check

[private]
cargo_clippy features mode: install_cargo_clippy
    cargo clippy --workspace --all-targets --exclude imap-codec-bench{{ features }}{{ mode }}

[private]
cargo_deny: install_cargo_deny
    cargo deny check

[private]
cargo_semver: install_cargo_semver_checks
    cargo semver-checks check-release --only-explicit-features -p imap-codec
    cargo semver-checks check-release --only-explicit-features -p imap-types

# Test multiple configurations
test: (test_impl ""           ""               ) \
      (test_impl ""           " --all-features") \
      (test_impl " --release" ""               ) \
      (test_impl " --release" " --all-features")

[private]
test_impl mode features: (cargo_test mode features)

[private]
cargo_test features mode:
    cargo test \
    --workspace \
    --exclude imap-types-fuzz \
    --exclude imap-codec-fuzz \
    --all-targets \
    --exclude imap-codec-bench\
    {{ features }}\
    {{ mode }}

# Audit advisories, bans, licenses, and sources
audit: cargo_deny

bench_check:
    cargo check -p imap-codec-bench --all-features --all-targets

# Benchmark
bench:
    cargo bench -p imap-codec-bench --all-features

# Benchmark against main
bench_against_main:
    rm -rf target/bench_tmp
    mkdir -p target/bench_tmp
    git clone --depth 1 https://github.com/duesee/imap-codec target/bench_tmp
    cd target/bench_tmp; cargo bench -p imap-codec-bench
    rm -rf target/criterion
    cp -r target/bench_tmp/target/criterion target/criterion
    cargo bench -p imap-codec-bench
    rm -rf target/bench_tmp

# Measure test coverage
coverage: install_rust_llvm_tools_preview install_cargo_grcov
    # Old build artifacts seem to be able to mess up coverage data (see #508),
    # removing everything in `target/coverage` seems to be the easiest fix for this.
    rm -rf target/coverage/*
    # Run instrumented tests to generate coverage information
    RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="$PWD/target/coverage/coverage-%m-%p.profraw" CARGO_TARGET_DIR="$PWD/target/coverage" cargo test -p imap-codec -p imap-types --all-features
    # Generate coverage reports
    # - LCOV info report for coveralls.io
    # - HTML report for local use
    grcov target/coverage \
        --source-dir . \
        --binary-path target/coverage/debug \
        --branch \
        --keep-only '{imap-codec/src/**,imap-types/src/**}' \
        --llvm \
        --output-types "html,lcov" \
        --output-path target/coverage/
    mv target/coverage/lcov target/coverage/coverage.lcov
    # Remove profiling information and build artifacts to prevent wasting disk space
    rm target/coverage/*.profraw
    rm -rf target/coverage/debug

# Fuzz all targets
[linux]
fuzz runs="25000": install_cargo_fuzz
    #!/usr/bin/env bash
    set -euo pipefail
    cd imap-codec
    for fuzz_target in $(cargo +nightly fuzz list)
    do
        echo "# Fuzzing ${fuzz_target}";
        cargo +nightly fuzz run --features=ext,arbitrary_simplified ${fuzz_target} -- -dict=fuzz/terminals.dict -max_len=256 -only_ascii=1 -runs={{ runs }};
    done



# Check MSRV
check_msrv: install_rust_msrv
    cargo '+{{ msrv }}' check --locked \
      --workspace --exclude imap-codec-bench \
      --all-targets --all-features 
    cargo '+{{ msrv }}' test --locked \
      --workspace --exclude imap-codec-bench --exclude imap-codec-fuzz --exclude imap-types-fuzz \
      --all-targets --all-features

# Check minimal dependency versions
check_minimal_dependency_versions: install_rust_nightly
    cargo +nightly update -Z minimal-versions
    cargo check \
      --workspace --exclude imap-codec-bench \
      --all-targets --all-features 
    cargo test \
      --workspace --exclude imap-codec-bench --exclude imap-codec-fuzz --exclude imap-types-fuzz \
      --all-targets --all-features
    cargo update

###############
### INSTALL ###
###############

# Install required tooling (ahead of time)
install: install_rust_msrv \
         install_rust_nightly \
         install_rust_nightly_fmt \
         install_rust_llvm_tools_preview \
         install_cargo_clippy \
         install_cargo_deny \
         install_cargo_fuzz \
         install_cargo_grcov \
         install_cargo_hack \
         install_cargo_semver_checks

[private]
install_rust_msrv:
    rustup toolchain install '{{ msrv }}' --profile minimal

[private]
install_rust_nightly:
    rustup toolchain install nightly --profile minimal

[private]
install_rust_nightly_fmt:
    rustup component add --toolchain nightly rustfmt

[private]
install_rust_llvm_tools_preview:
    rustup component add llvm-tools-preview

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
install_cargo_grcov:
    cargo install grcov

[private]
install_cargo_hack:
    cargo install --locked cargo-hack

[private]
install_cargo_semver_checks:
    cargo install --locked cargo-semver-checks
