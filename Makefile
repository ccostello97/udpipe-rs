.PHONY: build build-release \
        format-fix format-check lint-fix lint-check \
        docs docs-check audit msrv \
        test test-unit test-integration \
        coverage coverage-html coverage-lcov \
        clean fix check help

# === Building ===
build: ## Build debug binary
	cargo build

build-release: ## Build optimized release binary
	cargo build --release

# === Formatting ===
format-fix: ## Format code (Rust and C++)
	cargo +nightly fmt
	clang-format -i src/*.{cpp,h}

format-check: ## Check formatting (Rust and C++)
	cargo +nightly fmt --check
	clang-format --dry-run --Werror src/*.{cpp,h}

# === Linting ===
lint-fix: ## Auto-fix linter warnings
	cargo clippy --fix --all-targets --allow-dirty --allow-staged

lint-check: ## Check for linter warnings (Rust and C++)
	cargo clippy --all-targets -- -D warnings
	clang-tidy src/*.{cpp,h}

# === Documentation ===
docs: ## Build and open documentation
	cargo doc --no-deps --open

docs-check: ## Check documentation builds
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# === Security ===
audit: ## Security audit dependencies
	@command -v cargo-audit >/dev/null || cargo install --locked cargo-audit
	cargo audit

# === MSRV ===
msrv: ## Verify minimum supported Rust version
	@command -v cargo-msrv >/dev/null || cargo install --locked cargo-msrv
	cargo msrv verify

# === Testing ===
test: ## Run all tests
	cargo test

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests only
	cargo test --test '*'

# === Coverage ===
coverage: ## Show coverage summary (fail if <100% function coverage)
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@rustup component list --installed | grep -q llvm-tools-preview || rustup component add llvm-tools-preview
	cargo llvm-cov --fail-under-functions 100

coverage-html: ## Generate HTML coverage report (fail if <100% function coverage)
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@rustup component list --installed | grep -q llvm-tools-preview || rustup component add llvm-tools-preview
	cargo llvm-cov --html --open --fail-under-functions 100

coverage-lcov: ## Generate lcov.info for CI (fail if <100% function coverage)
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@rustup component list --installed | grep -q llvm-tools-preview || rustup component add llvm-tools-preview
	cargo llvm-cov --lcov --output-path lcov.info --fail-under-functions 100

# === Cleanup ===
clean: ## Remove build artifacts
	cargo clean

# === Combo targets ===
fix: format-fix lint-fix ## Fix all auto-fixable issues

check: format-check lint-check docs-check audit msrv test coverage-lcov ## Run all CI checks locally

# === Help ===
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-20s %s\n", $$1, $$2}'
