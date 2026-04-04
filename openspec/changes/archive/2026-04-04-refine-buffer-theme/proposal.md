## Why

`BufferTheme` mixes `String` (ANSI escape codes) and `ratatui::style::Color` fields, includes an unused `border_fg` field, and embeds compile-time constants (cursor mode codes, reset sequences) that aren't theme-configurable. The `from_enumeration` function accepts two individual ANSI strings instead of the theme, limiting extensibility for future per-entry styling. Additionally, the theme token registry contains dead constants: `COMMANDLINE_FG`, `COMMANDLINE_BG` (registered but never used in rendering), `CURSOR_NORMAL`, `CURSOR_INSERT` (defined but never registered), and `SYNTAX_THEME` (defined but never referenced).

## What Changes

- **BREAKING** Remove `border_fg` (unused ANSI string field) from `BufferTheme`
- **BREAKING** Convert all remaining ANSI-string color fields in `BufferTheme` to `ratatui::style::Color`: `cursor_line_bg`, `search_bg`, `line_nr`, `cur_line_nr_bold` (color portion)
- **BREAKING** Remove constant/non-theme fields from `BufferTheme`: `cursor_line_reset`, `cursor_normal_code`, `cursor_normal_reset`, `cursor_insert_code`, `cursor_insert_reset` — these become module-level constants in the buffer view
- Rename `border_fg_color`/`border_bg_color` to `border_fg`/`border_bg` (now the only border fields)
- Change `from_enumeration` to accept `&Theme` instead of two individual ANSI strings
- Add a Color-to-ANSI-fg and Color-to-ANSI-bg helper in `yeet-buffer` view internals for line rendering
- Remove unused token constants: `COMMANDLINE_FG`, `COMMANDLINE_BG`, `CURSOR_NORMAL`, `CURSOR_INSERT`, `SYNTAX_THEME` and their default color registrations

## Capabilities

### New Capabilities

### Modified Capabilities

- `buffer-theme-injection`: `BufferTheme` field types change from mixed String/Color to all `Color`, and constants are removed from the struct
- `theme-registry`: Unused token constants and their default registrations are removed
- `internal-refactor`: `from_enumeration` signature changes to accept `&Theme`

## Impact

- `yeet-buffer/src/lib.rs`: `BufferTheme` struct fields change types
- `yeet-buffer/src/view/mod.rs`: Use `Color`-based border fields, define cursor/reset constants
- `yeet-buffer/src/view/line.rs`: Convert `Color` fields to ANSI strings for line rendering
- `yeet-buffer/src/view/prefix.rs`: Convert `Color` fields to ANSI strings for prefix rendering
- `yeet-frontend/src/theme.rs`: Update `to_buffer_theme()` and `to_buffer_theme_with_border()` to produce `Color` values instead of ANSI strings
- `yeet-frontend/src/update/enumeration.rs`: Change `from_enumeration` to accept `&Theme`; remove ANSI string parameters from callers
