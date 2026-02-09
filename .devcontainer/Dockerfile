# syntax=docker/dockerfile:1.4

# Development container for udpipe-rs
# Includes all build tools, linters, and CI utilities - no local setup required

FROM rust:1.93.0-slim-bookworm

# Use bash with pipefail so piped commands fail properly
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# DEBIAN_FRONTEND=noninteractive - Suppress apt interactive prompts during install
# CC=clang-21                    - C compiler used by build.rs (via cc crate)
# CXX=clang++-21                 - C++ compiler used by build.rs for udpipe sources
# PATH="/usr/local/bin:${PATH}"  - Add /usr/local/bin to PATH
ENV DEBIAN_FRONTEND=noninteractive \
    CC=clang-21 \
    CXX=clang++-21 \
    PATH="/usr/local/bin:${PATH}"

# =========================================================================
# Install all system dependencies, LLVM toolchain, and Rust tools
# =========================================================================

# Install hadolint (pre-built binary from Docker Hub)
COPY --from=hadolint/hadolint:v2.14.0-debian /bin/hadolint /usr/local/bin/hadolint

RUN --mount=from=ghcr.io/astral-sh/uv,source=/uv,target=/bin/uv \
    --mount=from=golang:1.25.7-bookworm,source=/,target=/goroot \
    --mount=type=tmpfs,dst=/build \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    # Install prerequisites for adding LLVM repository
    apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates=20230311+deb12u1 \
        git=1:2.39.5-0+deb12u3 \
        gnupg=2.2.40-1.1+deb12u2 \
        wget=1.21.3-1+deb12u1 \
        lsb-release=12.0-1 \
        software-properties-common=0.99.30-4.1~deb12u1 \
    # Add LLVM repository
    && wget -qO - https://apt.llvm.org/llvm-snapshot.gpg.key \
        | gpg --dearmor -o /usr/share/keyrings/llvm-archive-keyring.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/llvm-archive-keyring.gpg] http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-21 main" \
        > /etc/apt/sources.list.d/llvm.list \
    # Install LLVM/Clang toolchain (clangd = C/C++ language server, lldb = debugger for CodeLLDB)
    && apt-get update && apt-get install -y --no-install-recommends \
        clang-21=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        clangd-21=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        clang-format-21=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        clang-tidy-21=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        libc++-21-dev=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        libc++abi-21-dev=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        lldb-21=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
        lld-21=1:21.1.5~++20251023083151+45afac62e373-1~exp1~20251023083333.51 \
    # Set unversioned binaries to LLVM 21 (so IDE and build find them)
    && update-alternatives --install /usr/bin/clang clang /usr/bin/clang-21 100 \
    && update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-21 100 \
    && update-alternatives --install /usr/bin/lldb lldb /usr/bin/lldb-21 100 \
    && update-alternatives --install /usr/bin/lldb-dap lldb-dap /usr/bin/lldb-dap-21 100 \
    && update-alternatives --install /usr/bin/clangd clangd /usr/bin/clangd-21 100 \
    && update-alternatives --install /usr/bin/clang-format clang-format /usr/bin/clang-format-21 100 \
    && update-alternatives --install /usr/bin/clang-tidy clang-tidy /usr/bin/clang-tidy-21 100 \
    && update-alternatives --install /usr/bin/lld lld /usr/bin/lld-21 100 \
    # Install yamllint (install tool via uv)
    && UV_TOOL_BIN_DIR=/usr/local/bin /bin/uv tool install "yamllint==1.35.1" \
    # Install yamlfmt (build from source)
    && GOROOT=/goroot/usr/local/go GOPATH=/build GOBIN=/usr/local/bin \
        /goroot/usr/local/go/bin/go install "github.com/google/yamlfmt/cmd/yamlfmt@v0.21.0" \
    # Remove temporary packages
    && apt-get purge -y --auto-remove wget gnupg software-properties-common \
    # Configure Rust toolchain (nightly for unstable features in CI)
    && rustup default nightly-2026-02-05 \
    && rustup component add clippy llvm-tools-preview miri rust-src rustfmt \
    # Install cargo-binstall
    && cargo install --locked cargo-binstall \
    # Install cargo tools via binstall
    && cargo binstall -y --locked \
        cargo-audit \
        cargo-deny \
        cargo-hack \
        cargo-llvm-cov \
        cargo-msrv \
        cargo-outdated \
        cargo-udeps \
        just

WORKDIR /workspace

CMD ["bash"]
