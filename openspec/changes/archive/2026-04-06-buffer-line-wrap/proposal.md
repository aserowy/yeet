## Why

Long lines in buffers (e.g., help pages, file previews) overflow the viewport and are only accessible via horizontal scrolling. This forces users to scroll right to read content, losing context of the line beginning. Vim solves this with `set wrap` / `set linebreak`, which soft-wraps lines at word boundaries within the viewport. Yeet should support the same behavior as a configurable option on the ViewPort.

## What Changes

- **ViewPort gets a `wrap` option**: When enabled, lines longer than the viewport content width are broken into multiple rendered lines. Breaking prefers word boundaries (spaces); words longer than the viewport are broken by character count.
- **Wrapped continuation lines have no line number or signs**: Only the first rendered line of a wrapped BufferLine gets signs, line number, and prefix. Continuation lines are indented to align with the content area.
- **`j`/`k` skip wrapped lines**: Vertical cursor movement (`j`, `k`, `gg`, `G`) operates on BufferLine indices, not visual lines. Moving down always goes to the next BufferLine, never into a continuation line.
- **`h`/`l`/`w`/`e`/`b` etc. traverse within wrapped lines**: Horizontal motions navigate the full BufferLine content as if it were one line. When the cursor crosses a wrap boundary, it visually moves to the next/previous rendered line within the same BufferLine. At the BufferLine boundary, `l` and `w` stop (existing behavior preserved).
- **Cursor line background spans all wrapped lines**: When the cursor is on a wrapped BufferLine, the cursor_line_bg is applied to all continuation lines of that BufferLine.
- **Viewport height calculation accounts for wrapping**: When wrap is enabled, the viewport must calculate how many terminal rows a BufferLine occupies to determine how many BufferLines fit on screen and which visual row corresponds to which BufferLine.

## Capabilities

### New Capabilities

- `buffer`: Configurable soft line wrapping at word boundaries for buffer content.

### Modified Capabilities

(none)

## Impact

- `yeet-buffer/src/model/viewport.rs`: Add `wrap: bool` field to ViewPort.
- `yeet-buffer/src/view/mod.rs`: `get_rendered_lines` and `get_styled_lines` must produce multiple rendered lines per BufferLine when wrapping.
- `yeet-buffer/src/view/line.rs`: `add_line_styles` must handle wrap segments — only the first segment gets cursor styling adjustments, but all segments get cursor_line_bg.
- `yeet-buffer/src/view/prefix.rs`: Continuation lines get empty prefix (spaces matching prefix width) instead of signs/line numbers.
- `yeet-buffer/src/update/viewport.rs`: `update_by_cursor` must account for wrapped line heights when determining scroll position and visible range.
- `yeet-buffer/src/model/ansi.rs`: May need a `split_at_char` or `take_chars` method for breaking content at wrap points.
- `yeet-frontend/src/update/command/help.rs`: Enable wrap on help buffer ViewPorts.
