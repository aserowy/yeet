## Context

The `Window` enum forms a binary tree where splits (`Horizontal`/`Vertical`) contain two child windows and a `SplitFocus` indicating which child is active. Leaf nodes are `Directory`, `QuickFix`, or `Tasks`. The function `close_focused_window_or_quit` currently operates on the root window of the current tab — it checks if the root is a split, then collapses it by keeping the unfocused side. This fails for nested splits because it always operates at the root level instead of traversing to the focused leaf's parent split.

## Goals / Non-Goals

**Goals:**
- Close only the innermost split containing the focused leaf window, preserving the rest of the tree
- When the root window is a leaf (no splits), continue to emit quit
- Correctly handle `discard_changes` for buffer IDs belonging to the dropped subtree

**Non-Goals:**
- Changing split creation or focus navigation behavior
- Modifying the `Window` enum structure
- Adding parent pointers or changing the tree to a non-recursive representation

## Decisions

**Recursive close on `Window` instead of operating at root level**

Rather than taking the root window and pattern matching once, implement a recursive method on `Window` that follows the focus path. At each split level:
- If the focused child is a leaf, replace the current split with the unfocused child (close the leaf)
- If the focused child is itself a split, recurse into it

This approach reuses the same focus-traversal pattern already used by `focused_window_mut()` and `focused_viewport()`. The method returns the dropped subtree so callers can handle buffer cleanup.

**Alternative considered: flatten + rebuild** — Walk the tree to find the focused leaf, remove it, and rebuild. Rejected because the binary tree with focus pointers already provides a natural traversal path, making this unnecessarily complex.

**Method lives on `Window`** — The close-focused logic is a tree operation, so it belongs as a method on `Window` (like `focused_window_mut`), called from `close_focused_window_or_quit`.

## Risks / Trade-offs

- [Recursive depth] The tree depth equals the number of nested splits. In practice this is small (< 10), so stack overflow is not a concern. → No mitigation needed.
- [Behavior change] Users who relied on the old (buggy) behavior of collapsing the root split may be surprised. → This is the correct fix; the old behavior was a bug.
