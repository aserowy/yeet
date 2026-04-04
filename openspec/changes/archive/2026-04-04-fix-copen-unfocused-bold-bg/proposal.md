## Why

When the copen window is unfocused, the bold-formatted current qfix entry has its buffer background reset after the `\x1b[0m` ANSI code. This is the same class of bug as the cursor-line-bg fix, but affecting two other code paths in `add_line_styles`/`add_cursor_styles` that don't replace embedded resets with buffer-bg-aware resets.

## What Changes

- Replace embedded `\x1b[0m` with `\x1b[0m` + buffer_bg in the `hide_cursor_line` branch of `add_cursor_styles` (cursor line when unfocused).
- Replace embedded `\x1b[0m` with `\x1b[0m` + buffer_bg in the non-cursor-line branch of `add_line_styles` that prepends buffer_bg.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: Buffer background must be preserved through ANSI resets on all lines, not just the focused cursor line.

## Impact

- `yeet-buffer/src/view/line.rs` — two code paths updated
