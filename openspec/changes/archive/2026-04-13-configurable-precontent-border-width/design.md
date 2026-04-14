## Context

The `ViewPort` struct controls buffer rendering layout. `get_precontent_border_width()` returns 1 when `get_precontent_width() > 0` (signs + line numbers + prefix column > 0), creating a 1-cell space between pre-content columns and the content area. This border is also subtracted a second time in `get_content_width()` to account for ratatui's `Block::inner()` with `Borders::RIGHT`.

The commandline viewport has `prefix_column_width: 1` for command count display but should not have a border, as it wastes space and creates an unintended visual gap.

## Goals / Non-Goals

**Goals:**
- Allow individual viewports to override the computed precontent border width
- Default behavior remains unchanged (Option is None → computed as before)
- Use the override on the commandline viewport to set border width to 0

**Non-Goals:**
- Changing the default border behavior for directory or content viewports
- Making this configurable via user-facing settings or Lua hooks (internal viewport field only)

## Decisions

### Decision 1: Use `Option<usize>` field on ViewPort

Add `precontent_border_width: Option<usize>` to `ViewPort`. In `get_precontent_border_width()`:
- If `Some(n)` → return `n` (the override)
- If `None` → return the computed value (1 if precontent > 0, else 0)

**Alternative considered**: A boolean `hide_precontent_border` flag. Rejected because `Option<usize>` is more flexible — it allows setting to any width, not just 0 or the default.

## Risks / Trade-offs

- [Risk] Callers could set `Some(1)` when precontent is 0, creating a border that doesn't correspond to any pre-content columns → Low risk since this is an internal field, not exposed to plugins or users
