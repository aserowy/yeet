## 1. Fix wrap_line to carry styles

- [x] 1.1 In `wrap_line` in `yeet-buffer/src/view/wrap.rs`, for each non-first segment, call `content.get_ansi_escape_sequences_till_char(offset)` to get the accumulated ANSI codes, then prepend them to the segment content

## 2. Tests

- [x] 2.1 Add a test verifying that a continuation segment of a red-styled line starts with the red ANSI code
- [x] 2.2 Add a test verifying that the first segment is unchanged (no extra prefix added)
- [x] 2.3 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
