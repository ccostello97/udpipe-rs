.DEFAULT_GOAL := help

CXX_GLOB := find src include -name '*.cpp' -o -name '*.h'
DOCKER_IMAGE := udpipe-rs-dev
DOCKER_RUN := docker run --rm -v $(CURDIR):/workspace -w /workspace $(DOCKER_IMAGE)

.PHONY: help
help:
	@echo "Usage: make [target]"
	@awk 'BEGIN {FS = ":.*##"} /^### / {printf "\n\033[1m%s\033[0m\n", substr($$0, 5)} /^[a-zA-Z_-]+:.*##/ {printf " %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

### Setup

.PHONY: docker
docker: ## Build the development Docker image
	@docker image inspect $(DOCKER_IMAGE) >/dev/null 2>&1 || docker build -t $(DOCKER_IMAGE) .

.PHONY: docker-rebuild
docker-rebuild: ## Force rebuild the Docker image
	docker build -t $(DOCKER_IMAGE) .

.PHONY: shell
shell: docker ## Open a shell in the Docker container
	docker run --rm -it -v $(CURDIR):/workspace -w /workspace $(DOCKER_IMAGE) bash

### Build

.PHONY: build
build: docker ## Compile the project in debug mode
	$(DOCKER_RUN) cargo build

.PHONY: docs
docs: docker ## Build and open API documentation
	$(DOCKER_RUN) cargo doc --no-deps
	open target/doc/udpipe_rs/index.html || xdg-open target/doc/udpipe_rs/index.html

### Fix

.PHONY: lint
lint: docker ## Auto-fix linter warnings
	-$(DOCKER_RUN) sh -c 'cargo clippy --fix --allow-dirty --allow-staged && $(CXX_GLOB) | xargs clang-tidy-18 --fix'

.PHONY: fmt
fmt: docker ## Auto-format code
	$(DOCKER_RUN) sh -c 'cargo fmt && $(CXX_GLOB) | xargs clang-format-18 -i'

.PHONY: fix
fix: lint fmt ## Apply all automatic fixes (lint + format)

### Check

.PHONY: lint-check
lint-check: docker ## Verify no linter warnings
	$(DOCKER_RUN) sh -c 'cargo clippy --all-targets --all-features -- -D warnings && $(CXX_GLOB) | xargs clang-tidy-18'

.PHONY: fmt-check
fmt-check: docker ## Verify code formatting
	$(DOCKER_RUN) sh -c 'cargo fmt -- --check && $(CXX_GLOB) | xargs clang-format-18 --dry-run --Werror'

.PHONY: docs-check
docs-check: docker ## Check documentation for warnings
	$(DOCKER_RUN) sh -c 'RUSTDOCFLAGS="-D warnings" cargo doc --no-deps'

.PHONY: audit
audit: docker ## Check dependencies for security vulnerabilities
	$(DOCKER_RUN) cargo audit

.PHONY: deny
deny: docker ## Check licenses and dependency bans
	$(DOCKER_RUN) cargo deny check

.PHONY: lockfile
lockfile: docker ## Verify lockfile is up-to-date
	$(DOCKER_RUN) cargo update --locked --workspace

.PHONY: unused-deps
unused-deps: docker ## Find unused dependencies
	$(DOCKER_RUN) cargo udeps --all-targets

.PHONY: outdated-deps
outdated-deps: docker ## Find outdated dependencies
	$(DOCKER_RUN) cargo outdated --exit-code 1

.PHONY: compat
compat: docker ## Verify minimum supported Rust version (MSRV)
	$(DOCKER_RUN) cargo msrv verify

.PHONY: test
test: docker ## Run tests
	$(DOCKER_RUN) cargo test

.PHONY: hack
hack: docker ## Test all feature combinations
	$(DOCKER_RUN) cargo hack test --feature-powerset --all-targets --workspace

.PHONY: bench
bench: docker ## Run benchmarks
	$(DOCKER_RUN) cargo bench

.PHONY: coverage
coverage: docker ## Run tests with coverage (enforces 100% function coverage)
	$(DOCKER_RUN) cargo llvm-cov --ignore-filename-regex 'vendor/.*' --fail-under-functions 100

.PHONY: coverage-lcov
coverage-lcov: coverage ## Generate LCOV coverage report
	$(DOCKER_RUN) cargo llvm-cov report --ignore-filename-regex 'vendor/.*' --lcov --output-path lcov.info

.PHONY: coverage-html
coverage-html: coverage ## Generate HTML coverage report and open in browser
	$(DOCKER_RUN) cargo llvm-cov report --ignore-filename-regex 'vendor/.*' --html
	open target/llvm-cov/html/index.html || xdg-open target/llvm-cov/html/index.html

### Sanitizers

.PHONY: asan
asan: docker ## Run tests with AddressSanitizer + UndefinedBehaviorSanitizer
	$(DOCKER_RUN) sh -c 'cargo clean && RUSTFLAGS="-Z sanitizer=address" cargo test --lib --tests --target $$(rustc -vV | grep host | cut -d" " -f2) && cargo clean'

.PHONY: tsan
tsan: docker ## Run tests with ThreadSanitizer + UndefinedBehaviorSanitizer
	$(DOCKER_RUN) sh -c 'cargo clean && RUSTFLAGS="-Z sanitizer=thread" cargo test -Zbuild-std --lib --tests --target $$(rustc -vV | grep host | cut -d" " -f2) -- --test-threads=1 && cargo clean'

.PHONY: sanitize
sanitize: asan tsan ## Run all sanitizer checks (ASAN, then TSAN)

### CI

.PHONY: check
check: lint-check fmt-check docs-check audit deny lockfile unused-deps outdated-deps compat hack bench coverage sanitize ## Run all checks

### Utilities

.PHONY: update
update: docker ## Update dependencies to latest compatible versions
	$(DOCKER_RUN) cargo update

.PHONY: clean
clean: ## Remove build artifacts
	rm -rf target

.PHONY: all
all: fix check ## Run all fixes followed by all checks
