## Why

`open.rs` calls `refresh_quickfix_buffer_in_window` directly on the current tab's window. This only refreshes copen in the current tab, missing copen buffers in other tabs. It should emit `Message::QuickFixChanged` instead, which triggers the cross-tab refresh.

## What Changes

- Replace both `refresh_quickfix_buffer_in_window` calls in `open.rs` with `Action::EmitMessages(vec![Message::QuickFixChanged])` appended to the returned actions.
- Change `refresh_quickfix_buffer_in_window` from `pub(crate)` to `fn` (private) since no external callers remain.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

_None. Same cross-tab refresh behavior, applied consistently._

## Impact

- `yeet-frontend/src/update/open.rs` — replace direct calls with emit
- `yeet-frontend/src/update/command/qfix/window.rs` — reduce visibility of helper
