# Contributing to udpipe-rs

## Getting Started

**Prerequisites:** Git, and either Docker (for Dev Containers) or a local Rust + LLVM toolchain.

### Recommended: Open in Dev Container

The project provides a [Dev Container](https://containers.dev/) with all dependencies (Rust nightly, Clang 21, clang-format, clang-tidy, cargo tools). No need to install anything locally.

1. **Clone with submodules** (required — C++ source is in `vendor/udpipe`):

   ```bash
   git clone --recurse-submodules https://github.com/ccostello97/udpipe-rs.git
   cd udpipe-rs
   ```

   If you already cloned without submodules: `git submodule update --init`

2. **Open in Dev Container**
   - **VS Code / Cursor:** Install the [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension, then use **“Reopen in Container”** from the command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`), or click “Reopen in Container” when prompted after opening the folder.
   - The container will build and open with the workspace at `/workspace`. All commands below run inside the container.

3. **Build and verify**

   ```bash
   just build
   just check
   just test
   ```

### Without Dev Container

If you prefer a local toolchain, you need Rust (nightly), Clang 21, `clang-format-21`, `clang-tidy-21`, and optionally `cargo-msrv`, `cargo-udeps`, `cargo-outdated`, `cargo-llvm-cov`, `cargo-hack`, `cargo-miri`. Then run the same `just` commands as above.

## Justfile Commands

Run `just` or `just --list` to see all commands.

### Build

| Command              | Description     |
|----------------------|-----------------|
| `just build`         | Build (debug)   |
| `just build-release` | Build (release) |

### Fix (before committing)

| Command         | Description                                   |
|-----------------|-----------------------------------------------|
| `just fix`      | **Apply all automatic fixes** (lint + format) |
| `just fix-lint` | Fix linter warnings (Clippy + clang-tidy)     |
| `just fix-fmt`  | Fix formatting (rustfmt + clang-format)       |

### Check (CI-style, no modifications)

| Command                    | Description                              |
|----------------------------|------------------------------------------|
| `just check`               | **Format + lint checks** (fast pre-push) |
| `just check-format`        | Check formatting only                    |
| `just check-lint`          | Check lints only                         |
| `just check-audit`         | Security audit                           |
| `just check-deny`          | Licenses and dependency bans             |
| `just check-lockfile`      | Lockfile up to date                      |
| `just check-deps-unused`   | Unused dependencies                      |
| `just check-deps-outdated` | Outdated dependencies                    |
| `just check-msrv`          | MSRV compatibility                       |

### Test

| Command                  | Description                    |
|--------------------------|--------------------------------|
| `just test`              | Unit and integration tests     |
| `just test-coverage`     | **Tests with coverage report** |
| `just test-all-features` | All feature combinations       |
| `just test-bench`        | Benchmarks                     |
| `just test-asan`         | AddressSanitizer + UBSAN       |
| `just test-tsan`         | ThreadSanitizer + UBSAN        |
| `just test-miri`         | Miri (undefined behavior)      |

### Docs & maintenance

| Command       | Description             |
|---------------|-------------------------|
| `just docs`   | Build API documentation |
| `just clean`  | Remove build artifacts  |
| `just update` | Update dependencies     |

## Code Standards

We use `rustfmt` (nightly) and Clippy (warnings as errors) for Rust, and `clang-format-21` and `clang-tidy-21` for C++. Run **`just fix`** to auto-fix formatting and linter issues.

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

Unit tests live in `src/lib.rs` under `#[cfg(test)]`. Integration tests in `tests/integration.rs` download real models and exercise the full pipeline. Benchmarks are in `benches/parse.rs`.

```bash
just test           # Run all tests
just test-coverage  # Tests with coverage (lcov + HTML)
just test-bench     # Benchmarks
```

## Before Submitting a PR

Run these inside the Dev Container (or your environment with the same tools):

1. **Auto-fix** so formatting and fixable lints are clean:

   ```bash
   just fix
   ```

2. **Checks** (what CI runs for format/lint):

   ```bash
   just check
   ```

3. **Tests and coverage:**

   ```bash
   just test-coverage
   ```

   Fix any failing tests or coverage regressions.

4. Optionally run the full check suite (audit, deny, lockfile, MSRV, etc.):

   ```bash
   just check check-audit check-deny check-lockfile check-msrv
   ```

5. Commit with [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, etc.) and open a PR against `main`.

## Pull Requests (summary)

1. Fork and clone with `--recurse-submodules`
2. **Open the repo in the Dev Container** (recommended) so all dependencies are available
3. Create a feature branch and make your changes
4. Run **`just fix`**, then **`just check`**, then **`just test`** and **`just test-coverage`**
5. Commit with [Conventional Commits](https://www.conventionalcommits.org/) and open a PR against `main`

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
