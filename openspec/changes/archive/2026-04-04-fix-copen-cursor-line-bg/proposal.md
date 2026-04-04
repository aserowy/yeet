## Why

When the `:copen` quickfix window is focused and the cursor is on the line matching `QuickFix.current_index`, the cursor line background resets to `buffer_bg` instead of displaying `cursor_line_bg`. This happens because the ANSI reset code (`\x1b[0m`) embedded in the bold-formatted current item line clears the background color set by the cursor line styling pass.

## What Changes

- Fix the ANSI-to-style conversion so that cursor line background is preserved when ANSI reset codes are encountered on the cursor line.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `quickfix`: The cursor line background must be maintained on the current qfix item line when the window is focused.

## Impact

- `yeet-buffer/src/view/line.rs` — cursor line background application interacts with ANSI style parsing
- `yeet-buffer/src/view/mod.rs` — ANSI-to-tui text conversion may need to respect a base background color
