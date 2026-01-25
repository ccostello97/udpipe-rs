# Pull Request

## Summary

Brief description of the changes.

## Related Issues

Fixes #

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Refactoring (no functional changes)
- [ ] Documentation
- [ ] CI/build changes

## Checklist

### Required

- [ ] `make fmt-check` passes
- [ ] `make lint-check` passes
- [ ] `make test` passes
- [ ] Documentation updated (if public API changed)

### If modifying Rust code

- [ ] No new `unsafe` blocks without `// SAFETY:` comments
- [ ] Thread safety considered (Model is `Send` but not `Sync`)

### If modifying C++ code (`src/`, `include/`)

- [ ] `make fmt` applied
- [ ] `make sanitize` passes

### If modifying FFI boundary

- [ ] Memory ownership is clear and documented
- [ ] Error handling preserves thread-local error messages
- [ ] `make sanitize` passes
