## 1. Refactor buffer.rs signatures

- [x] 1.1 Remove `buffer_theme` parameter from `render_window` and add `theme: &Theme` propagation to its recursive calls (it already has `theme`)
- [x] 1.2 Replace `buffer_theme: &yeet_buffer::BufferTheme` with `theme: &Theme` in `render_buffer_slot` signature, and create `BufferTheme` locally before each `buffer_view` / `render_directory_buffer` call
- [x] 1.3 Replace `buffer_theme: &yeet_buffer::BufferTheme` with `theme: &Theme` in `render_directory_buffer` signature, and create `BufferTheme` locally before the `buffer_view` call
- [x] 1.4 Remove `let buffer_theme = theme.to_buffer_theme();` from the top-level `view()` function and stop passing it to `render_window`

## 2. Verify

- [x] 2.1 Run `cargo build --workspace` and `cargo test --workspace` to confirm no regressions
