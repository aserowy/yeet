## 1. Fix cursor line background through ANSI resets

- [x] 1.1 In `add_cursor_styles` in `yeet-buffer/src/view/line.rs`, after prepending the cursor line background, replace embedded `\x1b[0m` sequences in the content with `\x1b[0m` followed by the cursor line background ANSI code so the background is re-applied after each reset

## 2. Tests

- [x] 2.1 Add a test in `yeet-buffer/src/view/mod.rs` that verifies cursor line background is preserved when the buffer line contains ANSI bold with a full reset (`\x1b[1m...\x1b[0m`)
- [x] 2.2 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
