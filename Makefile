.PHONY: all build build-release test format-fix format-check lint-fix lint-check \
        docs docs-check clean audit msrv coverage coverage-html coverage-lcov fix check help

# === Building ===
build: ## Build debug binary
	cargo build

build-release: ## Build optimized release binary
	cargo build --release

# === Testing ===
test: ## Run tests
	cargo test

coverage: ## Show coverage summary (fail if <100%)
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@rustup component list --installed | grep -q llvm-tools-preview || rustup component add llvm-tools-preview
	cargo llvm-cov --fail-under-lines 90

coverage-html: ## Generate HTML coverage report (fail if <100%)
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@rustup component list --installed | grep -q llvm-tools-preview || rustup component add llvm-tools-preview
	cargo llvm-cov --html --open --fail-under-lines 90

coverage-lcov: ## Generate lcov.info for CI (fail if <100%)
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@rustup component list --installed | grep -q llvm-tools-preview || rustup component add llvm-tools-preview
	cargo llvm-cov --lcov --output-path lcov.info --fail-under-lines 90

# === Formatting ===
format-fix: ## Format code
	cargo fmt

format-check: ## Check formatting (CI)
	cargo fmt --check

# === Linting ===
lint-fix: ## Auto-fix clippy warnings
	cargo clippy --fix --all-targets --allow-dirty --allow-staged

lint-check: ## Check for clippy warnings (CI)
	cargo clippy --all-targets -- -D warnings

# === Documentation ===
docs: ## Build and open documentation
	cargo doc --no-deps --open

docs-check: ## Check documentation builds (CI)
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# === Security & Compatibility ===
audit: ## Security audit dependencies
	@command -v cargo-audit >/dev/null || cargo install --locked cargo-audit
	cargo audit

msrv: ## Verify minimum supported Rust version
	@command -v cargo-msrv >/dev/null || cargo install --locked cargo-msrv
	cargo msrv verify

# === Utilities ===
clean: ## Remove build artifacts
	cargo clean

fix: format-fix lint-fix ## Fix all auto-fixable issues

check: format-check lint-check audit docs-check msrv test coverage-lcov ## Run all CI checks locally

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'
