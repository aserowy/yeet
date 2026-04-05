## 1. Update Enter handler

- [x] 1.1 Pass `&mut QuickFix` to `open::selected` and update `current_index` to the cursor's vertical index in the QuickFix match arm
- [x] 1.2 Call `refresh_quickfix_buffer` after updating `current_index` so the bold indicator moves to the selected entry

## 2. Tests

- [x] 2.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
