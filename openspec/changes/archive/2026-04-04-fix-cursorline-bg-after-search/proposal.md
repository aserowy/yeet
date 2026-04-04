## Why

When a search match appears on the cursor line, the ANSI reset sequence after the search highlight hardcodes the background back to `buffer_bg` instead of `cursor_line_bg`. This causes the cursor line background to visually break after any search match, reverting to the default background for the remainder of the line.

## What Changes

- Fix `add_search_styles()` to use the correct background color when resetting after a search match on the cursor line
- The reset sequence after a search highlight must be context-aware: use `cursor_line_bg` on the cursor line, `buffer_bg` elsewhere

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `buffer-rendering`: The search highlight reset must be aware of cursor line context so the background continues correctly after a match

## Impact

- `yeet-buffer/src/view/line.rs`: `add_search_styles()` needs cursor line context; `add_line_styles()` control flow may change
- `yeet-buffer/src/view/style.rs`: `ansi_reset_with_bg()` may need a contextual variant or the call site needs to pass the right color
- No API or dependency changes
