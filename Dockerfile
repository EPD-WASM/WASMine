FROM rust:1.81

RUN echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-18 main\ndeb-src http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-18 main" > /etc/apt/sources.list.d/llvm.list &&\
    wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - &&\
    apt-get update &&\
    apt-get install -y \
    llvm-18-dev \
    clang-18 \
    lld-18 \
    libpolly-18-dev \
    zlib1g-dev \
    libxml2-utils &&\
    apt-get clean all

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
