## Context

yeet is a Rust TUI file manager built on ratatui/crossterm. Colors are currently hardcoded in two distinct systems:

1. **Ratatui styles** — used for UI chrome (tabbar, statusline). Colors like `Color::LightBlue`, `Color::DarkGray` are passed directly to ratatui widgets in `yeet-frontend/src/view/`.
2. **Raw ANSI escape codes** — used in buffer content rendering (cursors, line numbers, signs, search highlights) in `yeet-buffer/src/view/`. These are injected as string literals (e.g., `\x1b[100m`) and later converted via `ansi_to_tui::IntoText`.
3. **Syntect themes** — syntax highlighting uses `ThemeSet::load_defaults()` with `"base16-eighties.dark"` hardcoded in `task/syntax.rs`.

There is no Lua runtime, no configuration file loading, and no color abstraction layer.

## Goals / Non-Goals

**Goals:**
- Embed a Lua 5.4 runtime that loads `~/.config/yeet/init.lua` at startup
- Expose a `y.theme` table allowing `y.theme.ColorName = '#rrggbb'` assignments
- Define a complete set of named color tokens covering all UI elements
- Replace every hardcoded color with a theme registry lookup
- Provide a sensible compiled-in default theme so yeet works without any config
- Make syntax highlighting theme configurable through the same system

**Non-Goals:**
- Runtime theme reloading (hot-reload) — restart required for theme changes
- Exposing non-theme Lua APIs (keybindings, commands, autocmds) — future work
- Shipping multiple built-in named themes (gruvbox, catppuccin, etc.) — users create their own
- Theming font/spacing/layout — only colors and text modifiers (bold, italic, underline)

## Decisions

### 1. Lua runtime: mlua with vendored Lua 5.4

**Choice**: Use the `mlua` crate with features `lua54` and `vendored`.

**Why over rlua**: mlua is actively maintained, supports Lua 5.4, has better async compatibility, and the vendored feature eliminates system Lua dependency. rlua is unmaintained.

**Why Lua 5.4 over LuaJIT**: LuaJIT is stuck at Lua 5.1 semantics and has platform limitations (no aarch64 until recently). For a config-only use case, JIT performance is irrelevant.

### 2. Theme registry as a flat token map

**Choice**: A single `Theme` struct containing a `HashMap<String, ResolvedColor>` where keys are dot-separated token names (e.g., `statusline.fg`, `tabbar.active.bg`) and values are resolved `ratatui::Color` values.

**Why flat map over nested structs**: A flat map is trivially extensible — adding a new token requires no struct changes. It maps directly to the Lua table path (`y.theme.statusline.fg`). The Lua-side nested table is flattened on read.

**Alternatives considered**: A strongly-typed struct with a field per token would give compile-time safety but requires updating the struct for every new token — poor extensibility for a user-facing API.

### 3. ANSI bridge: convert tokens to both ratatui::Style and ANSI strings

**Choice**: The theme registry provides two accessor methods:
- `style(token) -> ratatui::Style` — for UI chrome widgets
- `ansi(token) -> String` — for buffer content that uses ANSI escape codes

**Why**: The codebase has two rendering paths. Rather than rewriting the ANSI-based buffer rendering (which would be a massive refactor), we generate ANSI escape strings from the resolved colors. This keeps the change minimal.

### 4. Config file location: XDG Base Directory

**Choice**: Load from `$XDG_CONFIG_HOME/yeet/init.lua`, falling back to `~/.config/yeet/init.lua`.

**Why**: Follows XDG Base Directory Specification, which is standard for Linux CLI tools and consistent with neovim's approach.

### 5. Syntax highlighting: map syntect theme to a token name

**Choice**: Add a `y.theme.syntax` string token that selects a syntect built-in theme name (e.g., `"base16-ocean.dark"`). Individual syntax color tokens are a non-goal for this change.

**Why**: Syntect themes are complex (50+ color entries with scope selectors). Exposing individual syntax colors through the theme registry would require reimplementing syntect's theme system. Selecting from syntect's built-in set is pragmatic and covers the common case.

### 6. Thread the theme through the application

**Choice**: The `Theme` struct is created once at startup, stored in the application model, and passed by reference to all view functions. The `yeet-buffer` crate receives an opaque `BufferTheme` trait object (or a simple struct) to avoid coupling it to the full theme system.

**Why**: Avoids global state. The buffer crate should not depend on mlua or the full theme registry — it just needs the few color values relevant to buffer rendering.

## Risks / Trade-offs

**[Risk] mlua adds ~2-3MB to binary size and increases compile time** → Acceptable trade-off for Lua extensibility. The `vendored` feature avoids runtime dependency issues. This is a one-time cost that enables future Lua features beyond theming.

**[Risk] Flat token map has no compile-time validation of token names** → Mitigate with a `tokens` module that defines all token name constants as `&str`. Typos in Lua config fail silently (fall back to default) — document the full token list for users.

**[Risk] ANSI escape code generation from RGB colors requires 24-bit terminal support** → yeet already uses 24-bit ANSI codes in syntax highlighting (`as_24_bit_terminal_escaped`). Document terminal compatibility requirement. Fall back to 256-color approximation in the future if needed (non-goal for now).

**[Trade-off] No hot-reload means theme iteration requires restarting yeet** → Keeps implementation simple. Hot-reload can be added later by watching the config file with `notify` (already a dependency).

**[Trade-off] Syntect theme selection (not per-token syntax colors) limits syntax customization** → Pragmatic choice. Full syntax theming is complex and can be a follow-up change. Users can still choose from ~20 built-in syntect themes.
