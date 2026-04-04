## 1. Restructure add_line_styles control flow

- [x] 1.1 In `add_line_styles()`, compute `is_cursor_line` (and account for `hide_cursor_line`) before calling `add_search_styles()`
- [x] 1.2 Determine the effective background color: `cursor_line_bg` when on cursor line with highlighting active, `buffer_bg` otherwise

## 2. Pass background context to search styling

- [x] 2.1 Change `add_search_styles()` signature to accept a `bg: Color` parameter for the reset background
- [x] 2.2 Replace the hardcoded `style::ansi_reset_with_bg(theme.buffer_bg)` call with `style::ansi_reset_with_bg(bg)`
- [x] 2.3 Update the call site in `add_line_styles()` to pass the computed background color

## 3. Verify scenarios

- [x] 3.1 Search match on cursor line: background continues as `cursor_line_bg` after the match
- [x] 3.2 Search match on non-cursor line: background resets to `BufferBg` as before
- [x] 3.3 Multiple search matches on cursor line: `cursor_line_bg` between and after all matches
- [x] 3.4 Search match on cursor line with `hide_cursor_line`: background resets to `BufferBg`
