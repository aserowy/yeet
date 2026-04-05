## Why

When the focused window is a `topen` (Tasks) or `copen` (QuickFix) pane, the `:split` / `:vsplit` commands and their keybindings (`C-w C-s`, `C-w C-v`) fail because there is no current directory path to resolve. This is frustrating when the user wants to open a directory view alongside their tasks or quickfix list. The split command should fall back to the home directory when no directory path is available, and should also accept absolute paths and marks while continuing to reject relative paths in this context.

## What Changes

- When `:split` or `:vsplit` is executed without arguments and the focused window is a non-directory leaf (Tasks, QuickFix), fall back to the user's home directory as the split target instead of erroring.
- When `:split <path>` or `:vsplit <path>` is executed from a non-directory leaf, accept absolute paths and mark references (`'<char>`) as the target.
- Continue to error on relative paths when the focused window is a non-directory leaf, since there is no base directory to resolve them against.
- The `C-w C-s` and `C-w C-v` keybindings benefit automatically since they invoke `:split` / `:vsplit` with no arguments.

## Capabilities

### New Capabilities

### Modified Capabilities
- `window-management`: Split commands gain fallback behavior for non-directory focused windows (Tasks, QuickFix), accepting home directory default, absolute paths, and marks while rejecting relative paths.

## Impact

- `yeet-frontend/src/update/command/mod.rs`: Split/vsplit command handling logic for path resolution when current path is unavailable.
- `yeet-frontend/src/update/command/split.rs`: May need adjustment if split creation itself needs changes for non-directory source windows.
- `yeet-frontend/src/update/command/file.rs`: Path expansion may need a fallback for when no source directory exists.
