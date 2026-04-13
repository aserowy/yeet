## Context

The `get_line_number()` function in `yeet-buffer/src/view/prefix.rs` renders line numbers for each buffer line. It has three paths:

1. **Cursor line** (any line number mode): Uses `{:<width$}` format, producing exactly `width` visible characters (ANSI codes for bold/color around it don't count).
2. **Non-cursor, Absolute**: Uses `{:>width$} ` format (note trailing space), producing `width + 1` visible characters.
3. **Non-cursor, Relative**: Uses `{:>width$}` format, producing exactly `width` visible characters.

The Absolute non-cursor path is the only one with a trailing space, creating a 1-character column misalignment between the cursor line and all other lines.

## Goals / Non-Goals

**Goals:**

- Fix column alignment so all lines display content starting at the same column in absolute line number mode
- Ensure the fix is minimal and doesn't affect relative line number mode (which already works correctly)

**Non-Goals:**

- Changing the alignment direction (left vs right) of line numbers — this is intentional styling
- Refactoring the line number rendering to share code paths — separate fix
- Adding a separator between line numbers and content — that's a feature, not a bug fix

## Decisions

### Remove the trailing space from the absolute non-cursor line number format

**Decision**: Change `"{:>width$} "` to `"{:>width$}"` on the non-cursor absolute path in `get_line_number()`.

**Rationale**: This is the minimal fix. The trailing space is the only difference between absolute and relative non-cursor rendering. Removing it makes all three paths produce exactly `width` visible characters, which matches what `get_line_number_width()` returns and what the layout expects.

**Alternative considered**: Adding a trailing space to the cursor line path AND updating `get_line_number_width()` to return `width + 1` for absolute mode — rejected as more invasive for no benefit.

## Risks / Trade-offs

- **[Risk] Removing the space might make numbers feel cramped against content** → The relative mode already works without this space and looks fine. The cursor line in absolute mode also doesn't have the space. No visual regression expected. → Mitigation: existing tests for cursor line width already validate the layout is consistent.
