## Why

The help buffer's cursor line renders wider than the viewport, causing a visible line break that follows the cursor. This happens because `syntect`'s `LinesWithEndings` preserves trailing newlines in highlighted strings. When `ansi_to_tui::IntoText` converts these strings, it splits on the embedded `\n`, producing two `ratatui::Line`s per buffer line. The second (empty) line inherits cursor line background styling, creating the visual wrap artifact.

## What Changes

- Split highlighted strings on `\n` into separate `BufferLine`s when building the help buffer. Use `split_terminator('\n')` so that trailing newlines from `LinesWithEndings` produce exactly one `BufferLine` per source line, not an extra empty one.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `help-command-dispatch`: Each syntax-highlighted line SHALL produce exactly one `BufferLine`, regardless of embedded newlines.

## Impact

- `yeet-frontend/src/update/command/help.rs`: Change `BufferLine` construction to split on `\n` via `flat_map` with `split_terminator`.
