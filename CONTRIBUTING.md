# Contributing to udpipe-rs

## Getting Started

**Prerequisites:** Rust 1.85+, a C++11 compiler, and Git.

```bash
git clone --recurse-submodules https://github.com/ccostello97/udpipe-rs.git
cd udpipe-rs
make build && make test
```

> **Note:** The `--recurse-submodules` flag is required. UDPipe C++ source is vendored as a submodule in `vendor/udpipe`. If you forgot it, run `git submodule update --init`.

## Makefile Targets

Run `make help` to list all targets. The most useful:

```bash
make fix      # Auto-fix formatting and lint issues
make check    # Run full CI suite locally (required before PR)
make docs     # Build and open documentation
```

The `check` target runs: format check, clippy, doc build, security audit, MSRV verification, tests, and coverage (≥90% line coverage required).

## Code Standards

We use `rustfmt` (default settings) and `clippy` (warnings as errors). Run `make fix` to auto-fix issues.

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
make test              # All tests
make test-unit         # Unit tests only
make test-integration  # Integration tests only
make coverage-html     # Coverage report in browser
```

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
