# Justfile for build, check, and test
set shell := ["bash", "-uc"]

cxx_glob := "find src include \\( -name '*.cpp' -o -name '*.h' \\) -not -path '*/vendor/*'"
yaml_files := `{ echo .clangd .clang-format .clang-tidy .yamllint .yamlfmt; find . \( -name '*.yaml' -o -name '*.yml' \) -not -path './vendor/*' -not -path './.git/*' 2>/dev/null; } | tr '\n' ' '`

# Show available commands
default:
	just --list

# ------------------------------------------------------------------------------
# Build
# ------------------------------------------------------------------------------

# Build debug (default)
build:
	cargo build

# Build release
build-release:
	cargo build --release

# ------------------------------------------------------------------------------
# Fix (apply formatters / auto-fix linters)
# ------------------------------------------------------------------------------

# Fix linter warnings (Rust + C++)
fix-lint:
	cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
	{{cxx_glob}} | xargs clang-tidy-21 --fix

# Fix formatting (Rust + C++)
fix-fmt:
	cargo fmt
	{{cxx_glob}} | xargs clang-format-21 -i
	yamlfmt {{yaml_files}}

# Apply all automatic fixes
fix: fix-lint fix-fmt

# ------------------------------------------------------------------------------
# Check (CI-style checks, no modifications)
# ------------------------------------------------------------------------------

# Check lints only
check-lint:
	cargo clippy --all-targets --all-features -- -D warnings
	{{cxx_glob}} | xargs clang-tidy-21
	yamllint --strict {{yaml_files}}

# Check formatting only
check-format:
	cargo fmt -- --check
	{{cxx_glob}} | xargs clang-format-21 --dry-run --Werror
	yamlfmt --lint {{yaml_files}}

# Run format + lint checks (fast pre-push)
check: check-lint check-format

# Security audit
check-audit:
	cargo audit

# Cargo deny (licenses, bans)
check-deny:
	cargo deny check

# Lockfile up to date
check-lockfile:
	cargo update --locked --workspace

# Unused dependencies
check-deps-unused:
	cargo udeps --all-targets --all-features

# Outdated dependencies
check-deps-outdated:
	cargo outdated --exit-code 1

# MSRV compatibility
check-msrv:
	cargo msrv verify

# ------------------------------------------------------------------------------
# Test
# ------------------------------------------------------------------------------

# Unit and integration tests
test:
	cargo test

# All feature combinations
test-all-features:
	cargo hack test --feature-powerset --all-targets --workspace

# With coverage report
test-coverage:
	cargo llvm-cov --ignore-filename-regex='vendor/.*' -- --test-threads=1
	cargo llvm-cov report --ignore-filename-regex='vendor/.*' --lcov --output-path lcov.info
	cargo llvm-cov report --ignore-filename-regex='vendor/.*' --html

# Benchmarks
test-bench:
	cargo bench

# AddressSanitizer + UBSAN
test-asan: clean
	RUSTFLAGS="-Z sanitizer=address" cargo test --lib --tests --target $(rustc -vV | grep host | cut -d" " -f2)

# ThreadSanitizer + UBSAN
test-tsan: clean
	RUSTFLAGS="-Z sanitizer=thread" cargo test -Zbuild-std --lib --tests --target $(rustc -vV | grep host | cut -d" " -f2)

# Miri (undefined behavior)
test-miri:
	cargo miri test --lib

# ------------------------------------------------------------------------------
# Docs & maintenance
# ------------------------------------------------------------------------------

# Build API docs
docs:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Remove build artifacts
clean:
	cargo clean

# Update Cargo.lock / dependencies
update:
	cargo update
