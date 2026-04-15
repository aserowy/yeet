## 1. Remove pre-highlight truncation

- [x] 1.1 Remove `truncate_line()` function and all its tests from `yeet-frontend/src/task/syntax.rs`
- [x] 1.2 Update the highlighting loop in `highlight()` to pass the full original line to `highlighter.highlight_line()` instead of a truncated version
- [x] 1.3 Remove the `content_width` parameter from the `highlight()` function signature
- [x] 1.4 Update the call site in `yeet-frontend/src/task/mod.rs` to stop passing `rect.width` to `highlight()`

## 2. Add regression tests

- [x] 2.1 Add a test verifying that a line with a URL longer than a typical viewport width does not corrupt syntect parser state on subsequent lines (e.g., the README scenario: `src="https://..."` followed by a normal line should not be styled as string)

## 3. Verify and validate

- [x] 3.1 Run `cargo test` to ensure all tests pass
- [x] 3.2 Run `cargo clippy` and `cargo fmt` to ensure code is clean
- [x] 3.3 Run `git add -A && nix build .` to ensure the project builds successfully
