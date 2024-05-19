FROM rust:1.78

RUN apt-get update && apt-get install -y \
    clang \
    lld \
    libxml2-utils

RUN rustup component add \
    clippy \
    # required for coverage collection
    llvm-tools-preview &&\
    # required for test json output
    rustup install nightly

RUN cargo install \
    # util for conversion: cargo json output -> gitlab compatible reports
    gitlab-report \
    # required for coverage collection
    grcov
