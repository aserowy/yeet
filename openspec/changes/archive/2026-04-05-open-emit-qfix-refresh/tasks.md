## 1. Replace direct refresh calls in open.rs

- [x] 1.1 Replace both `refresh_quickfix_buffer_in_window` calls in `open.rs` with `Message::QuickFixChanged` emit in the returned actions
- [x] 1.2 Remove the `current_window_and_contents_mut` borrows that existed only for the refresh calls

## 2. Reduce helper visibility

- [x] 2.1 Change `refresh_quickfix_buffer_in_window` from `pub(crate)` to `fn` in `qfix/window.rs`

## 3. Verify

- [x] 3.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
