# Development container - all tools included, no local setup required
FROM rust:slim-trixie

ENV DEBIAN_FRONTEND=noninteractive \
    CXX=clang++-18

RUN \
    # Install system dependencies (clang-18 is in trixie's native repos)
    apt-get update && apt-get install -y --no-install-recommends \
        clang-18=1:18.1.8-18+b1 \
        clang-format-18=1:18.1.8-18+b1 \
        clang-tidy-18=1:18.1.8-18+b1 \
        libssl-dev=3.5.4-1~deb13u1 \
        pkgconf=1.8.1-4 \
    && rm -rf /var/lib/apt/lists/* \
    # Install nightly toolchain as default with all components
    && rustup default nightly \
    && rustup component add clippy llvm-tools-preview rust-src rustfmt \
    # Install cargo-binstall, then use it to install tools (pre-compiled binaries)
    && cargo install --locked cargo-binstall \
    && cargo binstall -y --locked \
        cargo-audit \
        cargo-deny \
        cargo-hack \
        cargo-llvm-cov \
        cargo-msrv \
        cargo-outdated \
        cargo-udeps \
    # Clean up cargo registry to reduce image size
    && rm -rf /usr/local/cargo/registry

WORKDIR /workspace

CMD ["bash"]
