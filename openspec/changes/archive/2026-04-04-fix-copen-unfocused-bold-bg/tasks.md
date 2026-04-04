## 1. Fix buffer background through ANSI resets

- [x] 1.1 In `add_cursor_styles` in `yeet-buffer/src/view/line.rs`, in the `hide_cursor_line` branch, replace embedded `\x1b[0m` with `\x1b[0m` + buffer_bg before appending padding
- [x] 1.2 In `add_line_styles`, in the non-cursor-line branch that prepends buffer_bg, replace embedded `\x1b[0m` with `\x1b[0m` + buffer_bg

## 2. Tests

- [x] 2.1 Add a test verifying buffer background is preserved on unfocused lines with ANSI bold content
- [x] 2.2 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
