## Context

`open::selected` in `open.rs` handles Enter on a copen entry. It reads the cursor position, finds the path, shifts focus to the directory, and emits `NavigateToPathAsPreview`. However, it never updates `QuickFix.current_index`, so the bold indicator (rendered by `build_qfix_lines`) remains on the old entry.

The `QuickFix` state lives on `state.qfix` in the update loop. The `open::selected` function currently only receives `app` — it does not have access to `state.qfix`.

## Goals / Non-Goals

**Goals:**
- Pressing Enter in copen sets `current_index` to the cursor line and refreshes the copen buffer bold indicator.

**Non-Goals:**
- Changing how other commands (`:cn`, `:cfirst`) update `current_index`.

## Decisions

**Pass `&mut QuickFix` to `open::selected` and update `current_index` there.**

In the QuickFix match arm, after resolving the path and before emitting the navigation action, set `qfix.current_index` to the cursor's vertical index and call `refresh_quickfix_buffer` to rebuild the bold-formatted lines.

This is the same pattern used by the quickfix command handlers (`:cn`, `:cfirst`, etc.) which update `current_index` and then call `refresh_quickfix_buffer`.

## Risks / Trade-offs

None. This follows the established pattern for quickfix index updates.
