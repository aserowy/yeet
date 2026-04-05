## Why

Closing a window when inside a nested split (e.g., a split within a split) closes the wrong level of the tree. The `close_focused_window_or_quit` function operates on the root window of the tab rather than recursively navigating to the focused leaf's parent split, causing the entire top-level split to collapse instead of just removing the focused pane.

## What Changes

- Fix `close_focused_window_or_quit` in `yeet-frontend/src/update/command/mod.rs` to recursively traverse the window tree following the focus path, closing only the innermost split that directly contains the focused leaf window
- When a focused leaf inside a nested split is closed, its parent split is replaced by the sibling subtree, preserving all other splits in the hierarchy
- When the focused window is the root (no splits), behavior remains unchanged (quit)

## Capabilities

### New Capabilities

- `window-management`: Defines the correct behavior for closing the currently focused window at any nesting depth in the split tree

### Modified Capabilities

## Impact

- `yeet-frontend/src/update/command/mod.rs` — `close_focused_window_or_quit` function (primary fix)
- `yeet-frontend/src/model/mod.rs` — may need a helper method on `Window` to support recursive close-at-focus
