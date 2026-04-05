## Why

There is no way to visually browse and interact with the quickfix list inside the editor. `:cl` prints entries to the command line but offers no persistent, navigable view. A split-based quickfix window (`:copen`) would allow users to see all entries, jump to them, and manage them without leaving the editing context.

## What Changes

- Add `:copen` command that opens a horizontal split window displaying all quickfix entries (like `:cl` format)
- The current quickfix entry (as tracked by `:cfirst`, `:cn`, `:cN`) is rendered bold
- Pressing `enter` on a selected entry opens its path in the nearest directory window (traverse the opposite side of the split containing the copen buffer to find a directory window)
- Pressing `dd` removes the selected entry from the quickfix list and updates the copen buffer
- Navigation uses the same keymaps as `:topen` (cursor movement within the buffer)
- All other keymaps besides topen navigation, `enter`, and `dd` are no-ops in the copen buffer
- `:cfirst`, `:cn`, `:cN` update the bold indicator in the copen buffer when it is open
- Add new `Window::QuickFix(ViewPort)` variant following the `Window::Tasks(ViewPort)` pattern

## Capabilities

### New Capabilities
- `copen-window`: The copen split window, its rendering, keymaps (enter, dd, topen-shared navigation), and interaction with the quickfix list and nearest directory window

### Modified Capabilities

## Impact

- `yeet-frontend/src/model/mod.rs`: New `Window::QuickFix` variant in the `Window` enum; update all match arms
- `yeet-frontend/src/model/qfix.rs`: Entry removal support
- `yeet-frontend/src/update/command/mod.rs`: Register `:copen` command
- `yeet-frontend/src/update/command/qfix/`: Restructured as module with `commands.rs` (quickfix commands) and `window.rs` (copen window logic)
- `yeet-frontend/src/update/qfix.rs`: Toggle/add with refresh for copen buffer
- `yeet-frontend/src/update/cursor.rs`: Enable j/k cursor movement for QuickFix and Tasks buffers
- `yeet-frontend/src/update/viewport.rs`: Enable gg/G viewport movement for QuickFix and Tasks buffers
- `yeet-frontend/src/view/statusline.rs`: Statusline rendering for QuickFix window
- `yeet-frontend/src/view/`: Copen buffer content rendering with bold for current entry
- `yeet-keymap/`: Keymap handling for copen buffer (enter, dd, navigation subset)
