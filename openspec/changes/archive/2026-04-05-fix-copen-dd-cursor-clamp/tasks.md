## 1. Fix cursor clamping in remove_entry

- [x] 1.1 In `remove_entry` in `yeet-frontend/src/update/command/qfix/window.rs`, change the out-of-bounds guard to clamp `cursor_index` to the last entry instead of returning early (keep early return only for empty entries)

## 2. Tests

- [x] 2.1 Add a test that verifies `dd` with cursor past the last entry clamps and removes the last entry
- [x] 2.2 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
