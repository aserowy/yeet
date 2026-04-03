## Why

yeet currently hardcodes all colors — ratatui `Style`/`Color` for UI chrome (tabbar, statusline) and raw ANSI escape codes for buffer content (cursors, line numbers, signs). The syntax highlighting theme (`base16-eighties.dark`) is also hardcoded. Users have no way to customize the visual appearance, which is a basic expectation for any terminal application, especially one inspired by neovim. A Lua-based theming system aligns with yeet's vision of Lua extensibility and gives users the same `y.theme.COLORNAME = '#001122'` workflow they know from neovim.

## What Changes

- **Add mlua dependency** to embed a Lua 5.4 runtime into yeet
- **Introduce a theme registry** — a centralized struct holding all named color tokens (e.g., `StatusLineFg`, `TabBarActiveBg`, `CursorLineBg`, `SignQfix`, `LineNr`, `Search`, etc.)
- **Load `init.lua` at startup** from `$XDG_CONFIG_HOME/yeet/init.lua` (or `~/.config/yeet/init.lua`), exposing `y.theme` as a Lua table that maps color token names to hex color strings
- **Replace all hardcoded colors** in `yeet-frontend` and `yeet-buffer` with lookups into the theme registry
- **Bridge syntect theme selection** to the theme registry so syntax highlighting colors are also configurable
- **Ship a single sensible default theme** compiled into the binary, used when no `init.lua` is present or when tokens are left unset

## Capabilities

### New Capabilities
- `lua-runtime`: Embed Lua 5.4 via mlua, load user `init.lua` at startup, expose the `y` global table
- `theme-registry`: Centralized color token registry populated from Lua config, with compiled-in defaults and fallback logic
- `theme-integration`: Replace all hardcoded colors across UI chrome and buffer rendering with theme registry lookups

### Modified Capabilities
<!-- No existing specs to modify -->

## Impact

- **New dependency**: `mlua` crate (with `lua54` and `vendored` features) added to workspace
- **yeet-frontend**: All view modules (`tabbar.rs`, `statusline.rs`, `commandline.rs`) and sign styling (`update/sign.rs`) refactored to use theme tokens instead of hardcoded colors
- **yeet-buffer**: `view/line.rs` and `view/prefix.rs` refactored to use theme tokens for cursor, search, and line number styling
- **yeet (main binary)**: Startup sequence extended to initialize Lua runtime and load `init.lua` before UI setup
- **Settings**: `settings.rs` extended to carry the resolved theme registry
- **Syntax highlighting**: `task/syntax.rs` updated to use theme-configured syntect theme or custom highlight colors
- **No breaking changes** to user-facing behavior — without an `init.lua`, yeet looks identical to today
