## 1. No-op proxy and custom searcher

- [x] 1.1 In `setup_and_execute`, after creating the `y.plugin` table, register a custom Lua package searcher that checks registered plugin names
- [x] 1.2 The searcher returns a loader function that creates a no-op proxy table (with `__index` returning no-op functions) for registered-but-not-loaded plugins
- [x] 1.3 The searcher returns nil for unknown modules (standard behavior)

## 2. Tests

- [x] 2.1 Add test: `require('plugin-name').setup()` does not error when plugin is registered but not loaded
- [x] 2.2 Add test: `require('unknown-module')` still errors as expected
- [x] 2.3 Verify existing `syntax_error_returns_none` test still passes

## 3. Build Verification

- [x] 3.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
