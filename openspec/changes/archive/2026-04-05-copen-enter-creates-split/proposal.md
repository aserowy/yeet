## Why

When the copen window is the only window in a tab (no sibling directory), pressing Enter does nothing — `open::selected` returns empty because `find_nearest_directory_in_sibling` returns `None`. The user expects Enter to always navigate to the selected entry, creating a directory window if needed.

## What Changes

- When Enter is pressed in copen and no sibling directory exists, create a horizontal split with a new directory window as the first child and copen as the second child, focus the directory window, and navigate to the selected entry's path.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: The "open entry in nearest directory window with enter" requirement needs to handle the case where no sibling directory exists by creating one.

## Impact

- `yeet-frontend/src/update/open.rs` — extend the QuickFix Enter handler to create a split when no directory sibling exists
