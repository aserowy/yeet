## 1. Update refresh_quickfix_buffer to iterate all tabs

- [x] 1.1 Change `refresh_quickfix_buffer` signature to take `tabs: &mut HashMap<usize, Window>` and `contents: &mut Contents` instead of a single window, iterate all tabs internally
- [x] 1.2 Extract current single-window logic into a private helper for use by `open.rs` and `remove_entry`

## 2. Move QuickFixChanged emit into qfix commands

- [x] 2.1 In `commands.rs`, add `Action::EmitMessages(vec![Message::QuickFixChanged])` to the returned actions of `select_first`, `next`, `previous`, `reset`, `clear_in`
- [x] 2.2 In `qfix.rs`, add the emit to `toggle` and `add`

## 3. Remove emit calls from callers

- [x] 3.1 In `command/mod.rs`, remove all 5 `QuickFixChanged` emit calls (cfirst, clearcl x2, cn, cN)
- [x] 3.2 In `update/mod.rs`, remove the 2 `QuickFixChanged` emit calls (FdResult/RgResult, ToggleQuickFix)

## 4. Verify Message::QuickFixChanged handler

- [x] 4.1 Keep `Message::QuickFixChanged` handler and variant — commands emit the message, handler processes it by calling `refresh_quickfix_buffer` across all tabs

## 5. Verify

- [x] 5.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
