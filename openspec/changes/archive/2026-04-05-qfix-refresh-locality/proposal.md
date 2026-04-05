## Why

The `Message::QuickFixChanged` emit is scattered across callers (`command/mod.rs`, `update/mod.rs`) rather than living inside the qfix commands that mutate the state. This violates locality of behavior — callers must remember to emit the message after every qfix mutation. Similarly, the cross-tab iteration loop lives in the `QuickFixChanged` handler rather than inside `refresh_quickfix_buffer`.

## What Changes

- Move `Message::QuickFixChanged` emit into the qfix commands themselves (`select_first`, `next`, `previous`, `reset`, `clear_in` in `commands.rs`, and `toggle`, `add` in `qfix.rs`).
- Change `refresh_quickfix_buffer` to accept `&mut HashMap<usize, Window>` (tabs) and `&mut Contents` so it iterates all tabs internally.
- Remove the `Message::QuickFixChanged` emits from `command/mod.rs` and `update/mod.rs`.
- Remove the `Message::QuickFixChanged` handler from `update_with_message`.
- Remove the `QuickFixChanged` variant from `Message` and its Debug impl.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

_None. This is a pure refactoring with no behavior change._

## Impact

- `yeet-frontend/src/update/command/qfix/commands.rs` — commands emit `QuickFixChanged`
- `yeet-frontend/src/update/qfix.rs` — `toggle` and `add` emit `QuickFixChanged`
- `yeet-frontend/src/update/command/qfix/window.rs` — `refresh_quickfix_buffer` iterates all tabs
- `yeet-frontend/src/update/command/mod.rs` — remove emit calls
- `yeet-frontend/src/update/mod.rs` — remove emit calls and handler
- `yeet-frontend/src/event.rs` — remove `QuickFixChanged` variant
