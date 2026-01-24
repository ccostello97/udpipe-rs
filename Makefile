.DEFAULT_GOAL := help

CXX_FILES := $(shell find src include -name '*.cpp' -o -name '*.h')

.PHONY: help
help:
	@echo "Usage: make [target]"
	@awk 'BEGIN {FS = ":.*##"} /^### / {printf "\n\033[1m%s\033[0m\n", substr($$0, 5)} /^[a-zA-Z_-]+:.*##/ {printf " %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

### Build

.PHONY: build
build: ## Compile the project in debug mode
	cargo build

.PHONY: docs
docs: ## Build and open API documentation
	cargo doc --open --no-deps

### Fix

.PHONY: lint
lint: dev ## Auto-fix linter warnings
	-cargo clippy --fix --allow-dirty --allow-staged
	-clang-tidy --fix $(CXX_FILES)

.PHONY: fmt
fmt: dev ## Auto-format code
	cargo +nightly fmt
	clang-format -i $(CXX_FILES)

.PHONY: fix
fix: lint fmt ## Apply all automatic fixes

### Check

.PHONY: lint-check
lint-check: dev ## Verify no linter warnings
	cargo clippy -- -D warnings
	clang-tidy $(CXX_FILES)

.PHONY: fmt-check
fmt-check: dev ## Verify code formatting
	cargo +nightly fmt -- --check
	clang-format --dry-run --Werror $(CXX_FILES)

.PHONY: type-check
type-check: ## Check for type errors
	cargo check --all-targets

.PHONY: docs-check
docs-check: ## Check documentation for warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

.PHONY: audit
audit: dev ## Check dependencies for security vulnerabilities
	cargo audit

.PHONY: deny
deny: dev ## Check licenses and dependency bans
	cargo deny check

.PHONY: lockfile
lockfile: ## Verify lockfile is up-to-date
	cargo update --locked --workspace

.PHONY: unused-deps
unused-deps: dev ## Find unused dependencies
	cargo +nightly udeps --all-targets

.PHONY: outdated-deps
outdated-deps: dev ## Find outdated dependencies
	cargo outdated --exit-code 1

.PHONY: compat
compat: dev ## Verify minimum supported Rust version (MSRV)
	cargo msrv verify

.PHONY: test
test: dev ## Run tests (without checking coverage)
	cargo test

.PHONY: hack
hack: dev ## Test all feature combinations
	cargo hack test --feature-powerset --all-targets --workspace

.PHONY: bench
bench: dev ## Run benchmarks
	cargo bench

.PHONY: coverage
coverage: dev ## Run tests and enforce 100% function coverage
	cargo llvm-cov --fail-under-functions 100
	cargo llvm-cov report --lcov --output-path lcov.info
	cargo llvm-cov report --html --open

.PHONY: check
check: lint-check fmt-check type-check docs-check audit deny lockfile unused-deps outdated-deps compat hack bench coverage ## Run all checks

### Utilities

.PHONY: dev
dev: ## Install required development tools
	@rustup component list --installed | grep -q clippy || rustup component add clippy
	@rustup component list --installed | grep -q rustfmt || rustup component add rustfmt
	@rustup component list --installed | grep -q llvm-tools || rustup component add llvm-tools-preview
	@rustup toolchain list | grep -q nightly || rustup toolchain add nightly
	@command -v cargo-audit >/dev/null || cargo install --locked cargo-audit
	@command -v cargo-deny >/dev/null || cargo install --locked cargo-deny
	@command -v cargo-msrv >/dev/null || cargo install --locked cargo-msrv
	@command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
	@command -v cargo-hack >/dev/null || cargo install --locked cargo-hack
	@command -v cargo-outdated >/dev/null || cargo install --locked cargo-outdated
	@command -v cargo-udeps >/dev/null || cargo +nightly install --locked cargo-udeps
	@clang-format --version 2>/dev/null | grep -q "version 18" || (echo "Error: clang-format 18 required (see CONTRIBUTING.md)" && exit 1)
	@clang-tidy --version 2>/dev/null | grep -q "version 18" || (echo "Error: clang-tidy 18 required (see CONTRIBUTING.md)" && exit 1)

.PHONY: update
update: ## Update dependencies to latest compatible versions
	cargo update

.PHONY: clean
clean: ## Remove build artifacts and caches
	cargo clean

.PHONY: all
all: fix check ## Run all fixes followed by all checks
