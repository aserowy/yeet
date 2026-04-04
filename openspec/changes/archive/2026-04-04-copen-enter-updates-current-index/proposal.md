## Why

When pressing Enter on a copen entry, the directory window navigates to that entry but `QuickFix.current_index` is not updated. The bold indicator in the copen buffer still highlights the previous current entry instead of the one just selected. This is confusing because the user expects Enter to also mark the entry as "current."

## What Changes

- Update `QuickFix.current_index` to match the cursor position when Enter is pressed in the copen window.
- Refresh the copen buffer lines so the bold indicator moves to the newly selected entry.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: The "open entry in nearest directory window with enter" requirement needs to include updating `current_index` and refreshing the bold indicator.

## Impact

- `yeet-frontend/src/update/open.rs` — update `current_index` in the Enter handler
- `yeet-frontend/src/update/command/qfix/window.rs` — refresh buffer after index change
