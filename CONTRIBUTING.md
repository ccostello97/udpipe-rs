# Contributing to udpipe-rs

## Getting Started

**Prerequisites:** Docker and Git.

```bash
git clone --recurse-submodules https://github.com/ccostello97/udpipe-rs.git
cd udpipe-rs
make docker  # Build the development container
make build   # Compile the project
```

> **Note:** The `--recurse-submodules` flag is required. UDPipe C++ source is vendored as a submodule in `vendor/udpipe`. If you forgot it, run `git submodule update --init`.

All development happens inside Docker containers, ensuring consistent tooling (Rust nightly, LLVM 18, cargo tools) across all platforms.

## Makefile Targets

Run `make help` to list all targets, organized by category:

### Setup

| Target           | Description                          |
| ---------------- | ------------------------------------ |
| `docker`         | Build the development Docker image   |
| `docker-rebuild` | Force rebuild the Docker image       |
| `shell`          | Open a shell in the Docker container |

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

| Target         | Description                                     |
| -------------- | ----------------------------------------------- |
| `lint-check`   | Check for linter warnings (Clippy + clang-tidy) |
| `fmt-check`    | Check code formatting (rustfmt + clang-format)  |
| `docs-check`   | Check documentation for warnings                |
| `audit`        | Check dependencies for security vulnerabilities |
| `deny`         | Check licenses and dependency bans              |
| `lockfile`     | Verify lockfile is up-to-date                   |
| `unused-deps`  | Find unused dependencies                        |
| `outdated-deps`| Find outdated dependencies                      |
| `compat`       | Verify minimum supported Rust version (MSRV)    |
| `test`         | Run tests                                       |
| `hack`         | Test all feature combinations                   |
| `bench`        | Run benchmarks                                  |
| `coverage`     | Run tests and enforce 100% function coverage    |
| `coverage-lcov`| Generate LCOV coverage report                   |
| `coverage-html`| Generate HTML coverage report and open          |
| `check`        | Run all checks (required before PR)             |

### Utilities

| Target   | Description                              |
| -------- | ---------------------------------------- |
| `update` | Update dependencies to latest compatible |
| `clean`  | Remove build artifacts                   |
| `all`    | Run all fixes followed by all checks     |

## Code Standards

We use `rustfmt` (nightly) and `clippy` (warnings as errors) for Rust, and `clang-format-18` and `clang-tidy-18` for C++. Run `make fix` to auto-fix issues.

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

Unit tests live in `src/lib.rs` under `#[cfg(test)]`. Integration tests in `tests/integration.rs` download real models and exercise the full pipeline. Benchmarks are in `benches/parsing.rs`.

```bash
make test      # Run all tests
make coverage  # Run tests with coverage enforcement
make bench     # Run benchmarks
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
