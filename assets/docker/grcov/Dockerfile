# docker run -v <path to imap-codec>:/opt/imap-codec -it <image>

from rust:latest

RUN cargo install grcov
RUN rustup component add llvm-tools-preview

WORKDIR /opt/imap-codec
ENV RUSTFLAGS="-Cinstrument-coverage"
ENV LLVM_PROFILE_FILE="coverage-%m-%p.profraw"

ENTRYPOINT ["/bin/bash"]

#cargo clean
#cargo test --workspace --all-features
#grcov . --source-dir . --binary-path target/debug -t html --branch -o target/debug/coverage --keep-only '{src/**,imap-types/src/**}'
