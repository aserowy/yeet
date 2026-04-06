## 1. Protect y table via _G metatable

- [x] 1.1 In `setup_and_execute`, store the `y` table in a Rust-side variable rather than setting it directly on `_G`
- [x] 1.2 Create a metatable for `_G` with `__index` that returns the protected `y` table when key is `"y"`
- [x] 1.3 Create `__newindex` on the `_G` metatable that intercepts `y = <table>` and shallow-merges into the existing `y` table; for non-`y` keys, use `rawset`
- [x] 1.4 Log a warning when `y` is assigned a non-table value (nil, string, number, etc.)

## 2. Tests

- [x] 2.1 Test: `y = { theme = { X = "val" } }` merges — `y.theme.X` is set and `y.hook` still exists with `:add()`
- [x] 2.2 Test: `y = { theme = { ... } }` followed by `y.hook.on_window_create:add(fn)` succeeds
- [x] 2.3 Test: `y = nil` does not destroy the `y` table
- [x] 2.4 Test: other global assignments (`foo = 42`) still work normally
- [x] 2.5 Test: existing theme and hook tests still pass (no regression)

## 3. Documentation

- [x] 3.1 Update `docs/help/configuration.md` to note that `y = { ... }` merges into the existing table

## 4. Validation

- [x] 4.1 Run `cargo test` across the full workspace
- [x] 4.2 Run `cargo clippy` and `cargo fmt` and fix any issues
- [x] 4.3 Run `markdownlint` on modified docs and fix any issues
