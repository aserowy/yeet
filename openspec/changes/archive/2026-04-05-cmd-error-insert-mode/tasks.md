## 1. Reproduce and verify bug

- [x] 1.1 Write a test that executes `:split <invalid-path>` and asserts the mode transitions back from Command to Normal/Navigation
- [x] 1.2 Write a test that executes `:vsplit <invalid-path>` and asserts the mode transitions back from Command to Normal/Navigation
- [x] 1.3 Write a test that executes `:split` with no resolvable current path and asserts mode transitions back
- [x] 1.4 Write a test that executes `:vsplit` with no resolvable current path and asserts mode transitions back

## 2. Fix error paths in split/vsplit

- [x] 2.1 Fix `split` early-return error path (path expansion failure) to route through `add_change_mode()` or `print_error()`
- [x] 2.2 Fix `split` match-arm error path (missing preview path) to route through `add_change_mode()` or `print_error()`
- [x] 2.3 Fix `vsplit` early-return error path (path expansion failure) to route through `add_change_mode()` or `print_error()`
- [x] 2.4 Fix `vsplit` match-arm error path (missing preview path) to route through `add_change_mode()` or `print_error()`

## 3. Verify

- [x] 3.1 Run `cargo test` and ensure all tests pass
- [x] 3.2 Run `cargo clippy` and `cargo fmt` and fix any issues
