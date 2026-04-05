## 1. Add path validation

- [x] 1.1 Add existence check for resolved target path in `split` handler, returning error if path does not exist
- [x] 1.2 Add existence check for resolved target path in `vsplit` handler, returning error if path does not exist

## 2. Tests

- [x] 2.1 Write test that `:split <non-existent-relative-path>` returns an error and does not create a split
- [x] 2.2 Write test that `:vsplit <non-existent-relative-path>` returns an error and does not create a split

## 3. Verify

- [x] 3.1 Run `cargo test` and ensure all tests pass
- [x] 3.2 Run `cargo clippy` and `cargo fmt` and fix any issues
