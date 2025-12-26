# Contributing to udpipe-rs

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.85+ (check with `rustc --version`)
- C++ compiler with C++11 support (for building UDPipe)
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/ccostello97/udpipe-rs.git
cd udpipe-rs

# Build the project (downloads and compiles UDPipe automatically)
cargo build

# Run tests
cargo test
```

## Development Workflow

We provide a `Makefile` for common tasks:

```bash
make help          # Show all available commands
make build         # Build debug binary
make test          # Run tests
make fmt           # Format code (alias: make format-fix)
make lint-check    # Check for clippy warnings
make fix           # Auto-fix formatting and lint issues
make check         # Run all CI checks locally
```

### Before Submitting a PR

Run the full CI check locally:

```bash
make check
```

This runs:
1. `format-check` — Verify code formatting
2. `lint-check` — Clippy lints with `-D warnings`
3. `audit` — Security vulnerability scan
4. `docs-check` — Documentation builds without warnings
5. `msrv` — Verify minimum supported Rust version
6. `test` — All tests pass
7. `coverage-lcov` — Generate coverage report

### Code Coverage

We aim for high test coverage. Generate a coverage report:

```bash
make coverage       # Summary in terminal
make coverage-html  # Open HTML report in browser
```

Note: Some FFI error paths cannot be tested as they require UDPipe internals to fail.

## Code Style

### Formatting

We use `rustfmt` with default settings:

```bash
cargo fmt           # Format all code
cargo fmt --check   # Check without modifying
```

### Linting

We use `clippy` with warnings as errors:

```bash
cargo clippy --all-targets -- -D warnings
```

### Documentation

All public items must have documentation:

```rust
/// Short description of the function.
///
/// Longer explanation if needed.
///
/// # Example
///
/// ```rust
/// // Example code here
/// ```
pub fn my_function() { }
```

Build and check docs:

```bash
make docs-check     # Check docs build without warnings
make docs           # Build and open docs
```

## Testing

### Unit Tests

Unit tests live in `src/lib.rs` in a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Integration Tests

Integration tests that require a real model live in `tests/integration.rs`. These download a model and test the full pipeline.

```bash
cargo test --test integration
```

## Pull Request Process

1. **Fork** the repository
2. **Create a branch** for your feature (`git checkout -b feature/amazing-feature`)
3. **Make your changes** and add tests
4. **Run `make check`** to verify everything passes
5. **Commit** with a descriptive message (see below)
6. **Push** to your fork
7. **Open a PR** against `main`

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new parsing option
fix: handle empty input correctly
docs: update README examples
test: add coverage for error paths
refactor: simplify model loading logic
chore: update dependencies
```

## Reporting Issues

When reporting bugs, please include:

- Rust version (`rustc --version`)
- Operating system
- Minimal reproduction code
- Expected vs actual behavior

## Questions?

Feel free to open an issue for questions or discussion.

