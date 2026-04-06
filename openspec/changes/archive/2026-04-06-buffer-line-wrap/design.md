## Context

The current rendering pipeline in `yeet-buffer` works as:

1. `get_rendered_lines`: `buffer.lines.skip(vertical_index).take(height)` — one BufferLine per terminal row.
2. `get_styled_lines`: For each line, build prefix (signs + line number + custom prefix + border) and content (with horizontal scroll via `skip_chars`).
3. Content is clamped to `content_width` and padded on the cursor line.

Horizontal scrolling currently uses `viewport.horizontal_index` to skip characters from line start. When wrap is enabled, horizontal scrolling is disabled — wrapping replaces it.

## Goals / Non-Goals

**Goals:**

- Soft-wrap long lines at word boundaries within the viewport content width.
- When a single word is longer than the viewport, fall back to character-count breaking.
- Preserve all existing behavior when `wrap` is `false` (default).
- `j`/`k` navigate between BufferLines, not visual lines.
- `h`/`l`/`w`/`e`/`b` traverse the full BufferLine, cursor visually crosses wrap boundaries.
- Cursor line background spans all visual lines of a wrapped BufferLine.
- Continuation lines get empty prefix (aligned indentation, no signs/numbers).

**Non-Goals:**

- `gj`/`gk` for visual-line movement within wraps (can be added later).
- Configurable wrap width different from viewport content width.
- Hard line wrapping (inserting actual newlines).

## Decisions

**Wrap logic lives in the view layer, not the model**

The BufferLine model stays unchanged — wrapping is purely a rendering concern. The view layer splits a single BufferLine into multiple `WrapSegment`s before styling. This keeps the model clean and cursor math simple.

Alternative considered: Storing wrapped lines in the model. Rejected because it would complicate every operation that touches buffer lines (undo, search, motions, etc.).

**Introduce `WrapSegment` struct for the view pipeline**

```
struct WrapSegment {
    content: Ansi,           // The segment's visible content
    is_first: bool,          // First segment gets prefix (signs, line number)
    buffer_line_index: usize // Which BufferLine this belongs to
    char_offset: usize,      // Character offset within the BufferLine
}
```

`get_rendered_lines` returns `Vec<BufferLine>` today. With wrapping, it needs to return something that tracks multiple visual lines per BufferLine. Rather than changing the return type, the wrapping split happens inside `get_styled_lines` — after selecting the BufferLines but before rendering.

**Word-boundary breaking algorithm**

For a line with `char_count > content_width`:
1. Walk the Ansi content character by character up to `content_width`.
2. Find the last space character at or before `content_width`. If found, break there (exclusive of the space — the space is consumed but not rendered at end of segment).
3. If no space is found (word longer than viewport), break at exactly `content_width`.
4. Repeat for the remaining content until exhausted.

This uses `Ansi::take_chars` and `Ansi::skip_chars` which already handle ANSI escape sequences correctly.

**Viewport height calculation with wrapping**

When `wrap: true`, `get_rendered_lines` can no longer simply `take(height)` BufferLines, because a single wrapped line may occupy multiple terminal rows. Instead:

1. Starting at `vertical_index`, iterate BufferLines.
2. For each BufferLine, calculate `visual_lines = ceil(char_count / content_width)` (simplified — actual calculation uses the word-break algorithm).
3. Accumulate visual lines until the total reaches or exceeds `height`.
4. Return those BufferLines (possibly truncating the last one's segments).

**Cursor position mapping**

The cursor's `horizontal_index` is an absolute character position within the BufferLine. To determine which visual row the cursor is on within a wrapped line:

- `visual_row = cursor_char_pos / content_width` (simplified — actual depends on where word breaks land).
- Within that row: `visual_col = cursor_char_pos - segment_start_offset`.

The `update_by_cursor` function needs to account for this when scrolling — if the cursor is on a wrapped line, the viewport must show enough rows to include the visual row containing the cursor.

**Horizontal scrolling is disabled when wrap is enabled**

When `wrap: true`, `viewport.horizontal_index` is forced to 0. The content is never horizontally scrolled — it wraps instead. This matches vim's behavior where `:set wrap` and horizontal scrolling are mutually exclusive.

## Risks / Trade-offs

- [Performance with very long lines] → The word-break algorithm is O(n) in line length. For extremely long lines (>10K chars), this could be measurable. Acceptable for the expected use case (help pages, normal files).
- [Viewport height estimation complexity] → Calculating how many BufferLines fit on screen requires pre-computing wrap counts for each line. This adds complexity to `get_rendered_lines` but is unavoidable for correct vertical scrolling.
- [Search highlighting across wrap boundaries] → Search match positions are in BufferLine character coordinates. The rendering must map these to the correct visual segment. The existing `search_char_position` vec is already in character-space, so this maps naturally.
- [Cursor line background across wrap segments] → All segments of a wrapped cursor line must get `cursor_line_bg`. The `is_cursor_line` check uses the BufferLine index, not the visual row, so this works naturally — all segments share the same `buffer_line_index`.
