## 1. Extract sanitization to shared module

- [x] 1.1 Create `yeet-frontend/src/task/sanitize.rs` with the `strip_non_sgr_escape_sequences` function and all its existing tests moved from `image.rs`
- [x] 1.2 Update `yeet-frontend/src/task/mod.rs` to declare the `sanitize` module
- [x] 1.3 Update `yeet-frontend/src/task/image.rs` to import and use `strip_non_sgr_escape_sequences` from the new `sanitize` module instead of its local definition

## 2. Add line truncation and sanitization to syntax highlighting

- [x] 2.1 Update `syntax::highlight()` signature to accept a `content_width: u16` parameter
- [x] 2.2 Truncate each line to `content_width` characters before passing it to `highlighter.highlight_line()`
- [x] 2.3 Apply `strip_non_sgr_escape_sequences` from the sanitize module to each highlighted line before pushing to the result vector
- [x] 2.4 Update the call site in `yeet-frontend/src/task/mod.rs` to pass `rect.width` as the `content_width` argument to `syntax::highlight()`

## 3. Verify and clean up

- [x] 3.1 Run `cargo test` to ensure all existing tests pass including moved sanitization tests
- [x] 3.2 Run `cargo clippy` and `cargo fmt` to ensure code quality
- [x] 3.3 Run `git add -A && nix build .` to ensure the project builds successfully
