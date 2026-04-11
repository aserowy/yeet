# Theme

## `y.theme`

The `y.theme` table is the global namespace for theme configuration. Assign hex color strings in `#rrggbb` format to override any theme token. Invalid color strings are silently ignored and the default value is used instead.

```lua
y = {
  theme = {
    TabBarActiveBg = "#87CEFA",
    StatusLineFocusedFg = "#FFFFFF",
    CursorLineBg = "#333333",
  }
}
```

## `syntax`

The `y.theme.syntax` value selects a syntect built-in theme for syntax highlighting in file previews. Set it to any valid syntect theme name. If unset or invalid, yeet defaults to `base16-eighties.dark`.

```lua
y = {
  theme = {
    syntax = "base16-ocean.dark",
  }
}
```

## Tabbar Tokens

### `TabBarActiveBg`

Background color of the currently active tab. Default: light blue.

### `TabBarActiveFg`

Foreground (text) color of the currently active tab. Default: black.

### `TabBarInactiveBg`

Background color of inactive tabs. Default: dark gray.

### `TabBarInactiveFg`

Foreground (text) color of inactive tabs. Default: white.

### `TabBarBg`

Background color of the tab bar area outside of tab labels. Default: black.

## Statusline Tokens

### `StatusLineFocusedFg`

Foreground color of the statusline label when the window is focused. The label is rendered in bold. Default: white.

### `StatusLineUnfocusedFg`

Foreground color of the statusline label when the window is not focused. Default: gray.

### `StatusLineBg`

Background color of the entire statusline area. Default: black.

### `StatusLinePositionFg`

Foreground color of the cursor position indicator (e.g., `3/42`) in the statusline. Default: gray.

### `StatusLineBorderBg`

Background color of the statusline border area between panes. Default: black.

### `StatusLineBorderFg`

Foreground color of the statusline border area between panes. Default: black.

### `StatusLinePermissionsFg`

Foreground color of the file permissions string in the directory statusline. Default: gray.

## Diff Tokens

### `DiffAdded`

Color for the `+N` added lines indicator in the directory statusline. Default: green.

### `DiffModified`

Color for the `~N` modified lines indicator in the directory statusline. Default: yellow.

### `DiffRemoved`

Color for the `-N` removed lines indicator in the directory statusline. Default: red.

## Buffer Tokens

### `BufferBg`

Background color of the buffer content area. Set this to change the main editing background. Default: terminal default (`Reset`).

### `CursorLineBg`

Background color of the line the cursor is on. This highlight helps visually track the current position. Default: `#808080` (medium gray).

### `SearchBg`

Background color for search match highlights. Active matches are rendered with this background when using `/` or `?` search. Default: red.

### `LineNr`

Foreground color for relative line numbers in the gutter. These numbers indicate distance from the cursor line. Default: `#808080` (medium gray).

### `CurLineNr`

Foreground color for the current (absolute) line number in the gutter. This highlights which line the cursor is on. Default: white.

### `BufferFileFg`

Default foreground color for file entries in directory buffers. When the `yeet-directory-icons` plugin is active, this token serves as the fallback color for file entries that do not match any icon class mapping. Default: white.

### `BufferDirectoryFg`

Default foreground color for directory entries in directory buffers. When the `yeet-directory-icons` plugin is active, this token serves as the fallback color for directory entries that do not match any icon class mapping. Distinct from `BufferFileFg` to allow independent styling of files and directories. Default: light blue.

## Border Tokens

### `DirectoryBorderFg`

Foreground color of borders between directory panes (parent, current, preview). Default: black.

### `DirectoryBorderBg`

Background color of borders between directory panes. Default: terminal default (`Reset`).

### `SplitBorderFg`

Foreground color of the vertical separator between split windows. Default: black.

### `SplitBorderBg`

Background color of the vertical separator between split windows. Default: terminal default (`Reset`).

## Sign Tokens

### `SignQfix`

Foreground color of the quickfix sign in the sign column. Entries in the quickfix list are marked with this color. Default: `#FF55FF` (bright magenta).

### `SignMark`

Foreground color of the mark sign in the sign column. Marked entries are indicated with this color. Default: `#55FFFF` (bright cyan).

## Icon Tokens

Icon and text colors in directory buffers are controlled by the `yeet-directory-icons` plugin, not by the core. The plugin registers `DirectoryIconsColor*` theme tokens for every file extension, filename, and directory name in its rule set. Without the plugin, directory entries are plain unstyled text.

### DirectoryIconsColor Tokens

The plugin defines a `DirectoryIconsColor*` token for each color category. During `setup()`, the plugin sets default values for all tokens — but only if the token is not already set by a theme plugin. During bufferline mutation, colors are resolved by reading these tokens from `y.theme`.

Token names follow the pattern `DirectoryIconsColor<Identifier>`:

| Token | Description | Default |
| --- | --- | --- |
| `DirectoryIconsColorDefaultFile` | Fallback color for unrecognized files | `#6d8086` |
| `DirectoryIconsColorDefaultDirectory` | Fallback color for unrecognized directories | `#8caaee` |
| `DirectoryIconsColorRs` | Rust source files (`.rs`) | `#dea584` |
| `DirectoryIconsColorLua` | Lua source files (`.lua`) | `#51a0cf` |
| `DirectoryIconsColorJs` | JavaScript files (`.js`, `.mjs`, `.cjs`) | `#cbcb41` |
| `DirectoryIconsColorTs` | TypeScript files (`.ts`) | `#519aba` |
| `DirectoryIconsColorPy` | Python files (`.py`) | `#ffbc03` |
| `DirectoryIconsColorGo` | Go source files (`.go`) | `#519aba` |
| `DirectoryIconsColorNix` | Nix files (`.nix`) | `#7ebae4` |
| `DirectoryIconsColorCargoToml` | `Cargo.toml`, `Cargo.lock` | `#dea584` |
| `DirectoryIconsColorDockerfile` | Dockerfiles and compose files | `#384d54` |
| `DirectoryIconsColorGitignore` | Git config files (`.gitignore`, `.gitmodules`, `.gitattributes`) | `#f14c28` |
| `DirectoryIconsColorDirGit` | `.git` directory | `#f14c28` |
| `DirectoryIconsColorDirGeneric` | Common directories (`src`, `lib`, `test`, `docs`, etc.) | `#6d8086` |

This is a representative subset. The full list of tokens is defined in the `yeet-directory-icons` plugin source. Every token can be overridden via `y.theme`.

### Theme Plugin Priority

When a theme plugin (e.g., `yeet-bluloco-theme`) sets `DirectoryIconsColor*` tokens before the icons plugin runs `setup()`, the icons plugin respects the theme-provided values and does not overwrite them. The icons plugin only uses its own built-in defaults when the theme has not set these tokens.

Load order determines priority: theme plugins loaded before `yeet-directory-icons` take precedence for all `DirectoryIconsColor*` tokens.

### Fallback Colors

When a file or directory does not match any specific icon rule in the plugin, the plugin falls back to:

- **`DirectoryIconsColorDefaultFile`** for unrecognized files
- **`DirectoryIconsColorDefaultDirectory`** for unrecognized directories

Override these tokens to change the fallback colors:

```lua
y.theme.DirectoryIconsColorDefaultFile = "#abb2bf"
y.theme.DirectoryIconsColorDefaultDirectory = "#3691ff"
```

Override individual file type colors:

```lua
y.theme.DirectoryIconsColorRs = "#ff6480"
y.theme.DirectoryIconsColorPy = "#f9c859"
```
