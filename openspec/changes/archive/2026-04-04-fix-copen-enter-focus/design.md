## Context

The copen Enter handler in `open::selected` (open.rs) checks for a sibling directory via `find_nearest_directory_in_sibling`, then emits `NavigateToPathAsPreview`. However, `NavigateToPathAsPreview` dispatches through the message loop and calls `navigate_to_path_with_selection`, which uses `get_focused_directory_viewports_mut` — a function that follows the current focus path. Since focus is still on the QuickFix window, no directory viewports are found, and navigation silently fails.

## Goals / Non-Goals

**Goals:**
- Enter on a copen entry navigates the sibling directory window to the entry's path.
- Focus moves from the QuickFix window to the sibling directory window.

**Non-Goals:**
- Changing how `NavigateToPathAsPreview` resolves viewports (that would affect other callers).

## Decisions

**Add a focus-shifting helper and call it before emitting the navigation message.**

In `open::selected`, after confirming a sibling directory exists, call a new helper `focus_nearest_directory` on `qfix/window.rs` that flips the `SplitFocus` from the QuickFix child to the sibling directory child. This mutates the window tree in-place so that when `NavigateToPathAsPreview` runs in the next message loop iteration, `get_focused_directory_viewports_mut` will follow focus to the directory window and succeed.

Alternative considered: Making `navigate_to_path_with_selection` accept explicit buffer IDs instead of following focus. Rejected because it changes a widely-used function's interface and would require passing IDs through the message system.

## Risks / Trade-offs

[Focus change order] Focus shifts before navigation completes, but since navigation is emitted as an action processed in the same update cycle, this is safe. → No mitigation needed.
