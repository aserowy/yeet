## Context

The earlier fix-copen-cursor-line-bg change added `replace_resets_with` in the `!hide_cursor_line` branch of `add_cursor_styles`. Two other paths in `line.rs` also prepend or rely on a background color but don't protect against embedded `\x1b[0m`:

1. `add_cursor_styles` when `hide_cursor_line = true` (unfocused cursor line): no bg is applied, so `\x1b[0m` resets to terminal default which may differ from buffer_bg.
2. `add_line_styles` for non-cursor lines above the cursor: buffer_bg is prepended but `\x1b[0m` in the content resets it.

## Goals / Non-Goals

**Goals:**
- Buffer background is preserved through ANSI resets on all lines in all focus states.

**Non-Goals:**
- Changing the focused cursor line fix (already done).

## Decisions

Apply the same `replace_resets_with` pattern using buffer_bg in both affected code paths.

## Risks / Trade-offs

None. Same proven pattern as the cursor line fix.
