## 1. Dependencies and Project Setup

- [x] 1.1 Add `mlua` (with `lua54`, `vendored` features) to workspace `Cargo.toml` and `yeet-frontend/Cargo.toml`
- [x] 1.2 Add `mlua` dependency to `yeet/Cargo.toml` (main binary crate) for Lua runtime initialization at startup

## 2. Theme Registry

- [x] 2.1 Create `yeet-frontend/src/theme.rs` module defining the `Theme` struct with a `HashMap<String, ratatui::style::Color>` and all default token values matching current hardcoded colors
- [x] 2.2 Implement `Theme::style(token) -> ratatui::Style` accessor that returns a Style from token lookup with fallback to default
- [x] 2.3 Implement `Theme::ansi(token) -> String` accessor that generates 24-bit ANSI escape code strings from resolved colors
- [x] 2.4 Implement hex color parsing (`#rrggbb` -> `Color::Rgb`) with validation and error logging for invalid values
- [x] 2.5 Define all token name constants in a `tokens` submodule (e.g., `STATUSLINE_FOCUSED_FG`, `TABBAR_ACTIVE_BG`, etc.)
- [x] 2.6 Create `BufferTheme` struct containing only buffer-relevant color values (cursor line, search, line numbers, signs, cursor styles) for the `yeet-buffer` crate

## 3. Lua Runtime Integration

- [x] 3.1 Create `yeet/src/lua.rs` module that initializes the mlua Lua 5.4 runtime and creates the global `y` table with a nested `theme` table
- [x] 3.2 Implement XDG config path resolution (`$XDG_CONFIG_HOME/yeet/init.lua` with `~/.config/yeet/init.lua` fallback)
- [x] 3.3 Implement `init.lua` loading with error handling — catch syntax and runtime errors, log them, and continue with defaults
- [x] 3.4 Implement reading `y.theme` table entries after script execution and populating the `Theme` struct from Lua values
- [x] 3.5 Implement `y.theme.syntax` string value reading for syntect theme selection

## 4. Thread Theme Through Application

- [x] 4.1 Store the resolved `Theme` in the application model (add field to the main app state struct)
- [x] 4.2 Pass `&Theme` to all `yeet-frontend` view functions (`view::tabbar`, `view::statusline`, `view::commandline`, `view::buffer`)
- [x] 4.3 Convert `Theme` to `BufferTheme` and pass it to `yeet-buffer` view functions
- [x] 4.4 Update `yeet/src/main.rs` startup sequence to initialize Lua runtime, load config, build Theme, and inject it into the app model before UI rendering

## 5. Replace Hardcoded Colors — UI Chrome

- [x] 5.1 Refactor `yeet-frontend/src/view/tabbar.rs` to use theme tokens for active tab bg/fg, inactive tab bg/fg, and tabbar background
- [x] 5.2 Refactor `yeet-frontend/src/view/statusline.rs` to use theme tokens for focused/unfocused fg, background, and diff indicator colors (green/yellow/red)
- [x] 5.3 Refactor `yeet-frontend/src/view/commandline.rs` to use theme tokens for foreground and background

## 6. Replace Hardcoded Colors — Buffer Content

- [x] 6.1 Refactor `yeet-buffer/src/view/line.rs` to use `BufferTheme` ANSI codes for cursor line background (`\x1b[100m`), search highlight (`\x1b[41m`), and cursor styles (inverse/underline)
- [x] 6.2 Refactor `yeet-buffer/src/view/prefix.rs` to use `BufferTheme` ANSI codes for current line number bold and relative line number gray
- [x] 6.3 Refactor `yeet-frontend/src/update/sign.rs` to use theme tokens for quickfix sign style (`\x1b[1;95m`) and mark sign style (`\x1b[1;96m`)

## 7. Syntax Highlighting Theme Integration

- [x] 7.1 Update `yeet-frontend/src/task/syntax.rs` to accept the syntect theme name from the `Theme` struct instead of hardcoding `"base16-eighties.dark"`
- [x] 7.2 Add fallback logic: if the configured theme name is not found in `ThemeSet::load_defaults()`, log an error and fall back to `"base16-eighties.dark"`

## 8. Testing and Validation

- [x] 8.1 Add unit tests for hex color parsing (valid, invalid, edge cases)
- [x] 8.2 Add unit tests for `Theme::style()` and `Theme::ansi()` accessors with default and custom values
- [x] 8.3 Add integration test that initializes Lua runtime, executes a theme script, and verifies the Theme struct is populated correctly
- [x] 8.4 Verify that yeet starts correctly with no `init.lua` present (defaults match current appearance)
- [x] 8.5 Verify that yeet starts correctly with a malformed `init.lua` (error logged, defaults used)
