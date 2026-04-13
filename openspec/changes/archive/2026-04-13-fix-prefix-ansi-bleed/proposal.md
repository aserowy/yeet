## Why

ANSI escape codes from prefix components (signs, line numbers) bleed into subsequent prefix components (prefix column icons) in the buffer view rendering. This causes terminal emulators to misrender nerdfont icons in the prefix column — the icons appear wider than they should because the preceding ANSI state (e.g., bold from cursor line numbers) affects how the terminal measures and draws Private Use Area glyphs. Without preceding ANSI codes (e.g., when line numbers are disabled), the icons render at their correct 1-cell width.

## What Changes

- Each prefix component (`get_signs`, `get_line_number`, `get_prefix_column`) will ensure it terminates its ANSI state with a proper reset so styles do not bleed into the next component
- The `get_border` function may also need a reset to ensure content starts clean
- All existing tests will be updated to verify ANSI state isolation between components

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `buffer`: Add requirement that prefix component ANSI styles must not bleed across component boundaries

## Impact

- `yeet-buffer/src/view/prefix.rs` — Update `get_signs`, `get_line_number`, `get_prefix_column`, and `get_border` functions to ensure proper ANSI resets at component boundaries
- `yeet-buffer/src/view/mod.rs` — Tests verifying line width and rendering consistency
