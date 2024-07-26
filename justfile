export RUSTFLAGS := "-D warnings"
export RUSTDOCFLAGS := "-D warnings"

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
        starttls,\
        ext_condstore_qresync,\
        ext_login_referrals,\
        ext_mailbox_referrals,\
        ext_id,\
        ext_metadata \
        --group-features \
        quirk_crlf_relaxed,\
        quirk_rectify_numbers,\
        quirk_missing_text,\
        quirk_id_empty_to_nil,\
        quirk_trailing_space\
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
        ext_login_referrals,\
        ext_mailbox_referrals,\
        ext_id,\
        ext_metadata\
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

# Build and test bindings
bindings: bindings_python

# Build and test Python bindings
bindings_python: install_python_black install_python_maturin install_python_mypy install_python_ruff
    # Remove any old wheels
    rm -rf target/wheels/*
    # Lint Python code using Black and Ruff
    python -m black --check bindings/imap-codec-python
    python -m ruff check bindings/imap-codec-python
    # Build Python extension
    cd bindings/imap-codec-python; maturin build --release
    # Install extension and run unit tests
    pip install --force-reinstall --find-links=target/wheels/ imap_codec
    cd bindings/imap-codec-python; python -m unittest -v
    # Perform static type checking using mypy
    python -m mypy bindings/imap-codec-python

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

# Check minimal dependency versions and MSRV
minimal_versions: install_rust_1_65 install_rust_nightly
    cargo +nightly update -Z minimal-versions
    cargo +1.65 check \
      --workspace --exclude tokio-client --exclude tokio-server --exclude imap-codec-bench \
      --all-targets --all-features 
    cargo +1.65 test \
      --workspace --exclude tokio-client --exclude tokio-server --exclude imap-codec-bench --exclude imap-codec-fuzz --exclude imap-types-fuzz \
      --all-targets --all-features
    cargo update

###############
### INSTALL ###
###############

# Install required tooling (ahead of time)
install: install_rust_1_65 \
         install_rust_nightly \
         install_rust_nightly_fmt \
         install_rust_llvm_tools_preview \
         install_cargo_clippy \
         install_cargo_deny \
         install_cargo_fuzz \
         install_cargo_grcov \
         install_cargo_hack \
         install_cargo_semver_checks \
         install_python_black \
         install_python_maturin \
         install_python_mypy \
         install_python_ruff

[private]
install_rust_1_65:
    rustup toolchain install 1.65 --profile minimal

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

[private]
install_python_black:
    python -m pip install -U black

[private]
install_python_maturin:
    python -m pip install -U maturin

[private]
install_python_mypy:
    python -m pip install -U mypy

[private]
install_python_ruff:
    python -m pip install -U ruff
