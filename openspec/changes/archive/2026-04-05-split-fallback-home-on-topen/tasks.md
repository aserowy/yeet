## 1. Path resolution for non-directory windows

- [x] 1.1 Add a fallback path resolution function that handles the case when `get_current_path` returns `None`: classify args as mark, absolute, relative, or empty and resolve accordingly (home dir for empty, mark lookup, absolute as-is, error for relative)
- [x] 1.2 Update the `split` command handler in `mod.rs` to use the fallback when `get_current_path` returns `None` instead of immediately erroring
- [x] 1.3 Update the `vsplit` command handler in `mod.rs` to use the same fallback when `get_current_path` returns `None`

## 2. Tests

- [x] 2.1 Test: split with no args from a non-directory window creates split targeting home directory
- [x] 2.2 Test: vsplit with no args from a non-directory window creates split targeting home directory
- [x] 2.3 Test: split with absolute path from a non-directory window creates split targeting that path
- [x] 2.4 Test: split with mark from a non-directory window creates split targeting the marked path
- [x] 2.5 Test: split with relative path from a non-directory window returns an error
- [x] 2.6 Test: split with non-existent absolute path from a non-directory window returns an error

## 3. Validation

- [x] 3.1 Run `cargo test` and ensure all tests pass
- [x] 3.2 Run `cargo clippy` and `cargo fmt` and fix any issues
