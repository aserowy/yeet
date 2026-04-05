## Context

When `:help <topic>` opens a help buffer to a specific heading, the code sets `vp.cursor.vertical_index = topic_match.line_offset`. The viewport is then adjusted by `update_by_cursor` during rendering, which only guarantees the cursor is visible — it may appear anywhere in the viewport (top, middle, or bottom depending on buffer size and viewport height).

The codebase already has `ViewPortDirection::TopOnCursor` which sets `viewport.vertical_index = cursor.vertical_index`, placing the cursor line at the top of the visible area. This is the exact behavior needed.

## Goals / Non-Goals

**Goals:**

- When `:help <topic>` resolves to a non-zero line offset, position the viewport so the matched heading is at the top of the visible area (zt behavior).

**Non-Goals:**

- Changing viewport behavior for bare `:help` (line 0 is already at the top).
- Changing how topic resolution works.
- Adding general zt-on-open behavior to other buffer types.

## Decisions

**Set viewport.vertical_index directly after cursor positioning**

After setting `vp.cursor.vertical_index = topic_match.line_offset`, also set `vp.vertical_index = topic_match.line_offset`. This is equivalent to what `ViewPortDirection::TopOnCursor` does and avoids needing to dispatch a separate viewport message.

Alternative considered: Emitting a `ViewPortDirection::TopOnCursor` action after opening the help buffer. Rejected because the viewport is directly accessible at the point where cursor is set, making a direct assignment simpler and avoiding an extra message dispatch cycle.

## Risks / Trade-offs

- [Viewport overwritten by render cycle] → `update_by_cursor` only scrolls if cursor is outside the visible range. Since both cursor and viewport top are set to the same line, the cursor is always visible and `update_by_cursor` will not override the position.
