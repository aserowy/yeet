## Why

When line numbers are displayed in absolute mode, the current cursor line's filename/content starts one character earlier than all other lines, breaking column alignment. This is because the non-cursor absolute line number rendering adds a trailing space (`"{:>width$} "`) producing `width + 1` visible characters, while the cursor line rendering produces only `width` visible characters. In relative mode, all lines produce consistent `width` characters and the bug does not manifest.

## What Changes

- Remove the trailing space from the non-cursor absolute line number rendering in `yeet-buffer/src/view/prefix.rs` so that all lines (cursor and non-cursor) produce exactly `line_number_width` visible characters for the line number column in absolute mode.

## Capabilities

### New Capabilities

### Modified Capabilities

- `buffer`: The implicit requirement that all lines in a buffer display with consistent column alignment needs an explicit scenario covering absolute line number mode.

## Impact

- `yeet-buffer/src/view/prefix.rs`: The `get_line_number` function's absolute non-cursor branch (line 71) has a trailing space that must be removed to match the cursor line width.
