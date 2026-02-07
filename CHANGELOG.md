# Changelog

## [1.0.0](https://github.com/ccostello97/udpipe-rs/compare/v0.1.8...v1.0.0) (2026-02-07)


### ⚠ BREAKING CHANGES

* Build and API changes require updates to workflows and dependency selection.

### Features

* add CI sanitizers, benchmarks, and coverage improvements ([#29](https://github.com/ccostello97/udpipe-rs/issues/29)) ([64b04db](https://github.com/ccostello97/udpipe-rs/commit/64b04dbbdcc227910e83a5b73bb64a872ad187e6))
* devcontainer, justfile, optional download, and streaming parser ([#35](https://github.com/ccostello97/udpipe-rs/issues/35)) ([75dc3fa](https://github.com/ccostello97/udpipe-rs/commit/75dc3fa3da2da902220b24616c1fd2cf979860f4))


### Bug Fixes

* enable llvm-header-guard lint ([877a40c](https://github.com/ccostello97/udpipe-rs/commit/877a40cd471e5ad0e86b70117539367c9d5cbd9b))
* enforce consistent llvm 18 clang tools ([81a90d6](https://github.com/ccostello97/udpipe-rs/commit/81a90d62a4d199eb812c6c35ba3cb835e5e8ef8f))
* improve makefile ([6fec4b6](https://github.com/ccostello97/udpipe-rs/commit/6fec4b626f4c23f1f6102c2b0f451e61388d4a88))
* update ci yaml ([1db96bd](https://github.com/ccostello97/udpipe-rs/commit/1db96bdbfacbc754a3b3f272d6aee610b2395fe2))

## [0.1.8](https://github.com/ccostello97/udpipe-rs/compare/v0.1.7...v0.1.8) (2025-12-28)


### Bug Fixes

* improve linter compliance ([c03a568](https://github.com/ccostello97/udpipe-rs/commit/c03a56800e86d910341e4c8f947f4f9966497226))

## [0.1.7](https://github.com/ccostello97/udpipe-rs/compare/v0.1.6...v0.1.7) (2025-12-28)


### Bug Fixes

* add clang-format and clang-tidy linting for C++ code ([8147ca4](https://github.com/ccostello97/udpipe-rs/commit/8147ca40bd905d5b60ea62fe07a78144b32abb46))

## [0.1.6](https://github.com/ccostello97/udpipe-rs/compare/v0.1.5...v0.1.6) (2025-12-28)


### Bug Fixes

* stream model downloads directly to disk instead of buffering in memory ([896207e](https://github.com/ccostello97/udpipe-rs/commit/896207e4285e8b178f59395b538865b1f7967c41))

## [0.1.5](https://github.com/ccostello97/udpipe-rs/compare/v0.1.4...v0.1.5) (2025-12-28)


### Bug Fixes

* add IDE tooling and project configuration ([ab8d842](https://github.com/ccostello97/udpipe-rs/commit/ab8d842352fd485bebd1d92b5173ccbf3e9066eb))
* configure codecov and add badges ([133a269](https://github.com/ccostello97/udpipe-rs/commit/133a269c8bed3177a098a57c651f0cbe3e680215))

## [0.1.4](https://github.com/ccostello97/udpipe-rs/compare/v0.1.3...v0.1.4) (2025-12-27)


### Bug Fixes

* make release workflow run after ci workflow ([90c0417](https://github.com/ccostello97/udpipe-rs/commit/90c04172a07f79c26288463f06ecdec3df16b30b))
