## Why

The theme system is missing a `BufferBg` token for the buffer background color, forcing it to always use the terminal default. Additionally, two tokens (`CommandLineFg`, `CommandLineBg`) are defined and carry default values but are never referenced anywhere in the codebase, adding dead code to the theme.

## What Changes

- Add a new `BUFFER_BG` theme token with a `Color::Reset` default (preserves current terminal-default behavior).
- Wire `BUFFER_BG` through `BufferTheme` so the buffer rendering respects the token.
- Remove the two unused token constants and their default color entries: `COMMANDLINE_FG`, `COMMANDLINE_BG`.
- Remove `to_buffer_theme()` convenience method; all call sites must use `to_buffer_theme_with_border(fg, bg)` to make border token selection explicit.
- Fix border token usage: directory panes use `DirectoryBorderFg`/`DirectoryBorderBg`, split panes use `SplitBorderFg`/`SplitBorderBg`, commandline uses `SplitBorderFg`/`SplitBorderBg`.
- Fix sign generation to use `SignQfix`/`SignMark` theme tokens instead of hardcoded ANSI codes.
- Update theme config parsing to accept the new token and stop accepting removed ones.

## Capabilities

### New Capabilities

- `buffer-background`: Introduce a themeable background color for the buffer area.

### Modified Capabilities

- `extended-theme-tokens`: Remove unused tokens from the token set and add `BufferBg`.
- `buffer-theme-injection`: Remove `to_buffer_theme()` convenience method; require explicit border token selection via `to_buffer_theme_with_border()`.
- `extended-theme-tokens`: Wire `SignQfix`/`SignMark` tokens into sign generation.

## Impact

- `yeet-frontend/src/theme.rs` — token definitions, defaults, and `to_buffer_theme` conversion.
- `yeet-buffer` crate — `BufferTheme` struct gains a `buffer_bg` field; rendering applies it.
- Theme config files (`.toml`) — `BufferBg` becomes a valid key; removed tokens are silently ignored if present.
- Tests in `theme.rs` and `yeet-buffer` that reference removed tokens or `BufferTheme` fields.
