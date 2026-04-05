## Why

When pressing `dd` in the copen window after the cursor has moved past the last qfix entry (e.g., after removing the last entry), `remove_entry` returns early because `cursor_index >= qfix.entries.len()`. The cursor stays stuck at an invalid position and subsequent `dd` presses do nothing. Instead, the cursor should be clamped to the last entry so the removal can proceed.

## What Changes

- In `remove_entry`, when `cursor_index >= qfix.entries.len()` and entries exist, clamp `cursor_index` to the last entry instead of returning early. Only return early if entries are empty.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: The "remove entry with dd" requirement needs to handle out-of-bounds cursor by clamping to last entry.

## Impact

- `yeet-frontend/src/update/command/qfix/window.rs` — `remove_entry` guard logic
