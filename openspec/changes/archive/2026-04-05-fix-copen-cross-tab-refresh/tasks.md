## 1. Fix buffer cleanup

- [x] 1.1 In `buffers::update` in `yeet-frontend/src/update/buffers.rs`, collect referenced buffer IDs from all tabs (`app.tabs.values()`) instead of only the current tab

## 2. Add QuickFixChanged message

- [x] 2.1 Add `QuickFixChanged` variant to `Message` in `yeet-frontend/src/event.rs`

## 3. Handle QuickFixChanged

- [x] 3.1 In `update_with_message` in `yeet-frontend/src/update/mod.rs`, handle `Message::QuickFixChanged` by iterating all tabs and calling `refresh_quickfix_buffer` for each window tree

## 4. Replace direct refresh calls with emit

- [x] 4.1 In `yeet-frontend/src/update/command/mod.rs`, replace all 5 direct `refresh_quickfix_buffer` calls (cfirst, clearcl x2, cn, cN) with emitting `Message::QuickFixChanged`
- [x] 4.2 In `yeet-frontend/src/update/mod.rs`, replace the 2 direct `refresh_quickfix_buffer` calls (FdResult/RgResult, ToggleQuickFix) with emitting `Message::QuickFixChanged`
- [x] 4.3 Keep direct `refresh_quickfix_buffer` calls in `open.rs` — they must run synchronously before window tree mutations

## 5. Tests

- [x] 5.1 Add a test in `buffers.rs` that verifies a QuickFix buffer in a non-current tab is not removed
- [x] 5.2 Add a test in `buffers.rs` that verifies a Tasks buffer in a non-current tab is not removed
- [x] 5.3 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
