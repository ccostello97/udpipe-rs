# Contributing to udpipe-rs

## Getting Started

**Prerequisites:** Rust 1.85+, a C++11 compiler, LLVM 18 (`clang-format`, `clang-tidy`), and Git.

```bash
git clone --recurse-submodules https://github.com/ccostello97/udpipe-rs.git
cd udpipe-rs
make dev    # Install development tools
make build  # Compile the project
```

> **Note:** The `--recurse-submodules` flag is required. UDPipe C++ source is vendored as a submodule in `vendor/udpipe`. If you forgot it, run `git submodule update --init`.

## Makefile Targets

Run `make help` to list all targets, organized by category:

### Build

| Target  | Description                       |
| ------- | --------------------------------- |
| `build` | Compile the project in debug mode |
| `docs`  | Build and open API documentation  |

### Fix

| Target | Description                      |
| ------ | -------------------------------- |
| `lint` | Apply automatic linter fixes     |
| `fmt`  | Apply automatic formatting fixes |
| `fix`  | Apply all automatic fixes        |

### Check

| Target       | Description                                     |
| ------------ | ----------------------------------------------- |
| `lint-check` | Check for linter warnings (Clippy + clang-tidy) |
| `fmt-check`  | Check code formatting (rustfmt + clang-format)  |
| `type-check` | Check for type errors                           |
| `docs-check` | Check documentation for warnings                |
| `audit`      | Check dependencies for security vulnerabilities |
| `compat`     | Verify minimum supported Rust version (MSRV)    |
| `coverage`   | Run tests and enforce 100% function coverage    |
| `check`      | Run all checks (required before PR)             |

### Utilities

| Target   | Description                              |
| -------- | ---------------------------------------- |
| `dev`    | Install required development tools       |
| `update` | Update dependencies to latest compatible |
| `clean`  | Remove build artifacts and caches        |
| `all`    | Run all fixes followed by all checks     |

## Code Standards

We use `rustfmt` (nightly) and `clippy` (warnings as errors) for Rust, and `clang-format` and `clang-tidy` (LLVM 18) for C++. Run `make fix` to auto-fix issues.

> **Note:** LLVM 18 is required for consistent formatting. Install via:
>
> - **macOS**: `brew install llvm@18`
> - **Ubuntu/Debian**: `apt install clang-format-18 clang-tidy-18`
> - **Other**: [LLVM releases](https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8)

All public items require documentation with examples where appropriate:

```rust
/// Parses the input text and returns tokens.
///
/// # Example
///
/// ```
/// let tokens = parse("Hello world");
/// ```
pub fn parse(input: &str) -> Vec<Token> { /* ... */ }
```

## Tests

Unit tests live in `src/lib.rs` under `#[cfg(test)]`. Integration tests in `tests/integration.rs` download real models and exercise the full pipeline.

```bash
make coverage  # Run all tests with coverage (opens HTML report)
```

The coverage target enforces 100% function coverage.

## Pull Requests

1. Fork and clone with `--recurse-submodules`
2. Create a feature branch
3. Make changes and add tests
4. Run `make check`
5. Commit with [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, etc.)
6. Open a PR against `main`

## Releases

Releases are automated via [release-please](https://github.com/googleapis/release-please). On every push to `main`, it analyzes commit messages and:

1. Creates or updates a **Release PR** that bumps the version in `Cargo.toml` and updates `CHANGELOG.md`
2. When the Release PR is merged, it creates a GitHub release and tag
3. The release triggers `cargo publish` to crates.io

This is why [Conventional Commits](https://www.conventionalcommits.org/) are required — release-please uses them to determine version bumps:

- `fix:` → patch (0.0.x)
- `feat:` → minor (0.x.0)
- `feat!:` or `BREAKING CHANGE:` → major (x.0.0)

## Issues

When reporting bugs, include: Rust version, OS, minimal reproduction, and expected vs actual behavior.
