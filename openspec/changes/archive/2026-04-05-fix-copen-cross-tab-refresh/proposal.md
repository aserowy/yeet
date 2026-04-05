## Why

Running quickfix commands (`:cfirst`, `:cn`, `:cN`, etc.) from a tab that doesn't contain the copen window breaks the copen buffer. After switching back to the tab with copen, the window is no longer rendered. Two bugs combine:

1. **Buffer cleanup removes cross-tab buffers**: `buffers::update` collects referenced buffer IDs only from the current tab (`app.current_window()`). When the current tab doesn't contain copen, the QuickFix buffer is considered stale and removed. The same issue affects `:topen` — the Tasks buffer is also removed when it lives in a non-current tab.
2. **Refresh only targets current tab**: All `refresh_quickfix_buffer` calls use `app.current_window_and_contents_mut()`, which only returns the current tab. Refreshing from another tab silently no-ops.

## What Changes

- Fix `buffers::update` to collect referenced buffer IDs from ALL tabs, not just the current tab. This fixes both copen and topen buffers being removed when they live in non-current tabs.
- Add a `QuickFixChanged` variant to `Message` in `yeet-frontend/src/event.rs`.
- Replace all direct `refresh_quickfix_buffer` calls (except inside `remove_entry` and `open`) with emitting `Action::EmitMessages(vec![Message::QuickFixChanged])`.
- Handle `Message::QuickFixChanged` in `update_with_message` by iterating all tabs and refreshing any quickfix buffer found.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: The "copen buffer refresh on quickfix mutation" requirement must work across tabs.

## Impact

- `yeet-frontend/src/event.rs` — new `QuickFixChanged` variant on `Message`
- `yeet-frontend/src/update/buffers.rs` — collect buffer IDs from all tabs
- `yeet-frontend/src/update/mod.rs` — handle `Message::QuickFixChanged`, replace direct refresh calls
- `yeet-frontend/src/update/command/mod.rs` — replace direct refresh calls with emit
