### Requirement: Tabbar uses theme colors
The tabbar view SHALL use theme tokens instead of hardcoded colors for active tab, inactive tab, and background styling.

#### Scenario: Custom tabbar colors
- **WHEN** `init.lua` sets `y.theme.TabBarActiveBg = '#0000ff'` and `y.theme.TabBarActiveFg = '#ffffff'`
- **THEN** the active tab renders with a blue background and white foreground

#### Scenario: Default tabbar colors match current appearance
- **WHEN** no tabbar theme tokens are set
- **THEN** the active tab uses light blue background with black foreground, and inactive tabs use dark gray background with white foreground

### Requirement: Statusline uses theme colors
The statusline view SHALL use theme tokens for all text and background colors, including focused/unfocused states, diff indicators, permissions foreground, and border background.

#### Scenario: Custom statusline colors
- **WHEN** `init.lua` sets `y.theme.StatusLineFocusedFg = '#00ff00'`
- **THEN** the focused statusline text renders in green

#### Scenario: Diff indicators use theme colors
- **WHEN** `init.lua` sets `y.theme.DiffAdded = '#00ff00'`, `y.theme.DiffModified = '#ffff00'`, `y.theme.DiffRemoved = '#ff0000'`
- **THEN** the statusline diff indicators render in the configured colors

#### Scenario: Permissions text uses theme color
- **WHEN** `init.lua` sets `y.theme.StatusLinePermissionsFg = '#ffaa00'`
- **THEN** the permissions string in the statusline renders with the configured color

#### Scenario: Statusline border background uses theme color
- **WHEN** `init.lua` sets `y.theme.StatusLineBorderBg = '#222222'`
- **THEN** the statusline border area background renders with the configured color

### Requirement: Buffer line rendering uses theme colors
The buffer line renderer SHALL use theme tokens for cursor line background, search highlight, cursor styling, and buffer entry foreground colors (file and directory) instead of hardcoded ANSI codes.

#### Scenario: Custom cursor line color
- **WHEN** `init.lua` sets `y.theme.CursorLineBg = '#333333'`
- **THEN** the current line background renders with the configured dark gray

#### Scenario: Custom search highlight
- **WHEN** `init.lua` sets `y.theme.SearchBg = '#ffaa00'`
- **THEN** search matches are highlighted with an orange background

#### Scenario: Directory entries use theme foreground color
- **WHEN** `init.lua` sets `y.theme.BufferDirectoryFg = '#00ff00'`
- **THEN** directory entries in the buffer render with green foreground instead of hardcoded bright blue

### Requirement: Line number rendering uses theme colors
The line number prefix renderer SHALL use theme tokens for current line number and relative line number styling.

#### Scenario: Custom line number colors
- **WHEN** `init.lua` sets `y.theme.LineNr = '#555555'` and `y.theme.CurLineNr = '#ffffff'`
- **THEN** relative line numbers render in dark gray and the current line number renders in white

### Requirement: Sign styling uses theme colors
Sign colors for quickfix and mark signs SHALL be configurable through theme tokens instead of hardcoded ANSI codes.

#### Scenario: Custom sign colors
- **WHEN** `init.lua` sets `y.theme.SignQfix = '#ff00ff'` and `y.theme.SignMark = '#00ffff'`
- **THEN** quickfix signs render in magenta and mark signs render in cyan

### Requirement: Syntax highlighting theme is configurable
The syntax highlighting task SHALL use the theme-configured syntect theme name instead of the hardcoded `"base16-eighties.dark"`.

#### Scenario: User selects a different syntax theme
- **WHEN** `init.lua` sets `y.theme.syntax = 'Solarized (dark)'`
- **THEN** file content preview uses Solarized dark syntax highlighting colors

### Requirement: Theme is threaded through the application without global state
The resolved `Theme` struct SHALL be passed by reference to all view functions. The `yeet-buffer` crate SHALL receive buffer-relevant theme values through a dedicated struct or trait, not the full theme. The `BufferTheme` struct SHALL include ratatui `Color` values for border foreground and background so the buffer view can use them with `Block::border_style()`.

#### Scenario: Buffer crate receives only its theme subset
- **WHEN** the buffer view function is called
- **THEN** it receives a `BufferTheme` containing cursor, search, line number, sign color values, and border color values — not the full theme registry

### Requirement: Commandline uses theme colors
The commandline view SHALL use theme tokens for foreground and background colors.

#### Scenario: Custom commandline colors
- **WHEN** `init.lua` sets `y.theme.CommandLineFg = '#cccccc'` and `y.theme.CommandLineBg = '#111111'`
- **THEN** the commandline renders with the configured colors
