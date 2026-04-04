## Context

`add_line_styles()` in `line.rs` calls `add_search_styles()` (line 20) before determining whether the current line is the cursor line (lines 22-27). Inside `add_search_styles()`, the reset after each search highlight is hardcoded to `theme.buffer_bg` (line 40). When the line happens to be the cursor line, `add_cursor_styles()` prepends `cursor_line_bg` — but the search reset sequences already baked in override it mid-line, producing a visible background break.

## Goals / Non-Goals

**Goals:**
- After a search match on the cursor line, the background continues as `cursor_line_bg` (not `buffer_bg`)
- Non-cursor lines are unaffected (still reset to `buffer_bg`)

**Non-Goals:**
- Refactoring the ANSI style system to a layered/composable model
- Changing how `\x1b[0m` full resets work globally
- Modifying cursor (caret) styling behavior

## Decisions

**Move cursor-line detection before search styling**

Currently `add_line_styles()` calls `add_search_styles()` first, then determines cursor line status. Reverse this: compute `is_cursor_line` upfront and pass the appropriate background color (`cursor_line_bg` or `buffer_bg`) into `add_search_styles()`.

Concretely:
1. In `add_line_styles()`, compute `cursor_line_offset` and `is_cursor_line` before calling `add_search_styles()`
2. Change `add_search_styles()` signature to accept a `bg: Color` parameter instead of deriving it from the theme
3. Call site passes `theme.cursor_line_bg` when on the cursor line, `theme.buffer_bg` otherwise

*Why this over alternatives:*
- **Passing `is_cursor_line` bool**: Leaks cursor-line concern into search styling. A `Color` parameter is more general — search styling just needs to know "what background to reset to".
- **Post-processing replacements**: Fragile — would need to find and replace specific ANSI sequences in the string after the fact. Error-prone if escape sequence formats change.
- **Reordering to apply cursor bg after search**: Would require cursor bg to wrap around search highlights without being reset by them — same fundamental problem.

## Risks / Trade-offs

**Early return path changes** → The current early return at line 24 (`None => return ansi`) happens before search styling. After this change, search styling runs before that return, meaning non-visible lines also get search styled. This is fine — the early return is for lines where the cursor offset can't be computed (cursor above viewport), and search styling on those lines is correct behavior regardless.

**`hide_cursor_line` path** → When `vp.hide_cursor_line` is true in `add_cursor_styles()`, the cursor line bg is not applied. The caller should pass `buffer_bg` in this case, not `cursor_line_bg`. This needs to be handled at the call site in `add_line_styles()`, by checking `vp.hide_cursor_line` when deciding which bg to pass.
