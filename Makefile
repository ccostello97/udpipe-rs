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

.PHONY: compat
compat: dev ## Verify minimum supported Rust version (MSRV)
	cargo msrv verify

.PHONY: test
test: dev ## Run tests (without checking coverage)
	cargo test

.PHONY: coverage
coverage: dev ## Run tests and enforce 100% function coverage
	cargo llvm-cov --fail-under-functions 100
	cargo llvm-cov report --lcov --output-path lcov.info
	cargo llvm-cov report --html --open

.PHONY: check
check: lint-check fmt-check type-check docs-check audit compat coverage ## Run all checks

### Utilities

.PHONY: dev
dev: ## Install required development tools
	rustup component add clippy rustfmt llvm-tools-preview
	cargo install --locked cargo-audit cargo-msrv cargo-llvm-cov
	clang-format --version 2>/dev/null | grep -q "version 18" || (echo "Error: clang-format 18 required (see CONTRIBUTING.md)" && exit 1)
	clang-tidy --version 2>/dev/null | grep -q "version 18" || (echo "Error: clang-tidy 18 required (see CONTRIBUTING.md)" && exit 1)

.PHONY: update
update: ## Update dependencies to latest compatible versions
	cargo update

.PHONY: clean
clean: ## Remove build artifacts and caches
	cargo clean

.PHONY: all
all: fix check ## Run all fixes followed by all checks
