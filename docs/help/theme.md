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

Icon color tokens are defined by the `yeet-directory-icons` plugin, not by the core. The plugin maps file extensions and directory names to icon classes, each with a default color. Override any icon class token via `y.theme`:

```lua
y = {
  theme = {
    -- Override the Rust icon color
    IconRust = "#E57373",
    -- Override the directory icon color
    IconDirectory = "#42A5F5",
  }
}
```

Token names depend on the plugin. Refer to the `yeet-directory-icons` plugin documentation for the full list of icon class tokens.

When a file or directory does not match any icon class, the plugin falls back to `BufferFileFg` (for files) or `BufferDirectoryFg` (for directories). Override these tokens to change the fallback colors.
