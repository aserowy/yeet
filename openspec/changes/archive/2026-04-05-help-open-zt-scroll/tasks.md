## 1. Viewport positioning on help open

- [x] 1.1 In `help::open` in `yeet-frontend/src/update/command/help.rs`, after setting `vp.cursor.vertical_index = topic_match.line_offset`, also set `vp.vertical_index = topic_match.line_offset` to position the matched heading at the top of the viewport

## 2. Tests

- [x] 2.1 Write a test verifying that when `:help <section>` opens to a non-zero line offset, the viewport `vertical_index` equals the cursor `vertical_index` (zt positioning)
- [x] 2.2 Write a test verifying that bare `:help` opens with both cursor and viewport at line 0
- [x] 2.3 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
