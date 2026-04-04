## Why

Pressing Enter on a copen entry does not navigate the directory window to that entry's path, and focus stays on copen instead of moving to the directory window. The root cause is that `open::selected` emits `NavigateToPathAsPreview`, which uses `get_focused_directory_viewports_mut` to find the directory viewports. Since focus is still on the QuickFix window, this function returns `None` and the navigation silently fails.

## What Changes

- In the QuickFix Enter handler (`open::selected`), shift focus from the QuickFix window to the sibling directory window before emitting the navigation message, so that `NavigateToPathAsPreview` can find the directory viewports and the user ends up focused on the directory.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: The "open entry in nearest directory window with enter" requirement needs to include focus transfer to the directory window and successful directory navigation.

## Impact

- `yeet-frontend/src/update/open.rs` — change the QuickFix Enter handler to shift focus before navigation
- `yeet-frontend/src/update/command/qfix/window.rs` — potentially add a helper to shift focus from QuickFix to sibling directory
