### Requirement: Theme struct holds all color tokens
The system SHALL maintain a `Theme` struct that maps named color tokens to resolved color values. Each token represents a single foreground color, background color, or text modifier for a specific UI element.

#### Scenario: Theme is populated from Lua assignments
- **WHEN** `init.lua` contains `y.theme.StatusLineFg = '#ff5500'`
- **THEN** the `Theme` struct contains the token `StatusLineFg` mapped to RGB(255, 85, 0)

#### Scenario: Theme provides defaults for unset tokens
- **WHEN** a token is not set in `init.lua`
- **THEN** the `Theme` struct returns the compiled-in default color for that token

### Requirement: Color token names cover all UI elements
The system SHALL define color tokens for at minimum the following UI elements:
- Tabbar: active tab foreground/background, inactive tab foreground/background, tabbar background
- Statusline: foreground, background (focused and unfocused variants), diff added/modified/removed colors, permissions foreground, border foreground, border background
- Buffer: cursor line background, search highlight background, line number foreground, current line number foreground, sign column foreground, file entry foreground, directory entry foreground
- Directory window: border foreground, border background
- Split: border foreground, border background
- Signs: quickfix sign color, mark sign color

The system SHALL NOT define token constants for UI elements that have no rendering code consuming them. Specifically, `CommandLineFg`, `CommandLineBg`, `CursorNormal`, `CursorInsert`, and `syntax` (as a token constant) SHALL NOT exist as constants or default registrations until rendering code uses them.

#### Scenario: All hardcoded colors have corresponding tokens
- **WHEN** the default theme is loaded with no user overrides
- **THEN** the rendered UI is visually identical to the current hardcoded appearance

#### Scenario: No dead token constants
- **WHEN** inspecting the `tokens` module
- **THEN** every constant is referenced by at least one rendering or theme conversion function outside of the `Default` impl

### Requirement: Theme provides ratatui Style accessors
The system SHALL provide a method to retrieve a `ratatui::Style` value for any token, suitable for use in ratatui widget rendering.

#### Scenario: Style retrieval for UI chrome
- **WHEN** the tabbar view requests the style for `TabBarActiveBg`
- **THEN** the theme returns a `ratatui::Style` with the configured background color

### Requirement: Theme provides ANSI escape code accessors
The system SHALL provide a method to retrieve an ANSI escape code string for any token, suitable for injection into ANSI-styled buffer content.

#### Scenario: ANSI retrieval for buffer rendering
- **WHEN** the line renderer requests the ANSI code for `CursorLineBg`
- **THEN** the theme returns a string like `\x1b[48;2;r;g;bm` representing the 24-bit background color

### Requirement: Hex color string parsing
The system SHALL parse color values from `init.lua` as hex strings in the format `#rrggbb` (case-insensitive). Invalid color strings SHALL be ignored, and the default color for that token SHALL be used.

#### Scenario: Valid hex color
- **WHEN** `init.lua` sets `y.theme.StatusLineFg = '#1a2b3c'`
- **THEN** the token resolves to RGB(26, 43, 60)

#### Scenario: Invalid hex color
- **WHEN** `init.lua` sets `y.theme.StatusLineFg = 'not-a-color'`
- **THEN** the token falls back to its default value and an error is logged

### Requirement: Syntect theme selection
The system SHALL support a `y.theme.syntax` string value that selects a syntect built-in theme by name. If unset or invalid, the system SHALL use `"base16-eighties.dark"` as the default.

#### Scenario: Valid syntect theme name
- **WHEN** `init.lua` sets `y.theme.syntax = 'base16-ocean.dark'`
- **THEN** syntax highlighting uses the `base16-ocean.dark` syntect theme

#### Scenario: Invalid syntect theme name
- **WHEN** `init.lua` sets `y.theme.syntax = 'nonexistent-theme'`
- **THEN** syntax highlighting falls back to `base16-eighties.dark` and an error is logged

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

### Requirement: Statusline permissions foreground is theme-configurable
The system SHALL apply the `StatusLinePermissionsFg` theme token to file permission text in the statusline.

#### Scenario: Custom permissions color
- **WHEN** `init.lua` sets `y.theme.StatusLinePermissionsFg = '#ffaa00'`
- **THEN** the permissions string in the statusline renders with orange foreground

#### Scenario: Default permissions color
- **WHEN** no `StatusLinePermissionsFg` token is set
- **THEN** permissions text renders with gray foreground

### Requirement: Statusline border background is theme-configurable
The system SHALL apply the `StatusLineBorderBg` theme token to the background of the statusline border area.

#### Scenario: Custom statusline border background
- **WHEN** `init.lua` sets `y.theme.StatusLineBorderBg = '#222222'`
- **THEN** the statusline border area renders with dark gray background

#### Scenario: Default statusline border background
- **WHEN** no `StatusLineBorderBg` token is set
- **THEN** the statusline border background renders as black

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

### Requirement: Buffer entry foreground colors are theme-configurable
The system SHALL apply theme-derived foreground colors to file and directory entries in directory buffers. File entries SHALL use the `BufferFileFg` token. Directory entries SHALL use the `BufferDirectoryFg` token. No hardcoded ANSI color codes SHALL remain for directory entry styling.

#### Scenario: Custom directory entry color
- **WHEN** `init.lua` sets `y.theme.BufferDirectoryFg = '#00ff00'`
- **THEN** directory entries in the buffer render with green foreground

#### Scenario: Custom file entry color
- **WHEN** `init.lua` sets `y.theme.BufferFileFg = '#cccccc'`
- **THEN** file entries in the buffer render with light gray foreground

#### Scenario: Default directory color matches current appearance
- **WHEN** no `BufferDirectoryFg` token is set
- **THEN** directory entries render with light blue foreground (matching current hardcoded `\x1b[94m`)

#### Scenario: Default file color
- **WHEN** no `BufferFileFg` token is set
- **THEN** file entries render with white foreground

### Requirement: BufferBg token exists in the token set
The system SHALL define a `BUFFER_BG` constant in the `tokens` module with the string value `"BufferBg"`. The `Theme::default()` implementation SHALL insert a default value of `Color::Reset` for this token.

#### Scenario: BufferBg token default
- **WHEN** a default theme is created
- **THEN** `theme.color(tokens::BUFFER_BG)` returns `Color::Reset`

#### Scenario: BufferBg token override
- **WHEN** `init.lua` sets `y.theme.BufferBg = '#282828'`
- **THEN** `theme.color(tokens::BUFFER_BG)` returns `Color::Rgb(40, 40, 40)`

### Requirement: Line number rendering uses theme colors
The line number prefix renderer SHALL use theme tokens for current line number and relative line number styling.

#### Scenario: Custom line number colors
- **WHEN** `init.lua` sets `y.theme.LineNr = '#555555'` and `y.theme.CurLineNr = '#ffffff'`
- **THEN** relative line numbers render in dark gray and the current line number renders in white

### Requirement: Sign colors use theme tokens
The sign generation function SHALL use the `SignQfix` and `SignMark` theme tokens for sign styling. No hardcoded ANSI color codes SHALL remain in the sign generation code. The `generate_sign` function SHALL accept a `&Theme` parameter.

#### Scenario: Qfix sign uses theme token color
- **WHEN** a qfix sign is generated with a theme where `SignQfix` is `Color::Rgb(255, 85, 255)`
- **THEN** the sign's style contains the ANSI escape for that color (`\x1b[38;2;255;85;255m`)

#### Scenario: Mark sign uses theme token color
- **WHEN** a mark sign is generated with a theme where `SignMark` is `Color::Rgb(85, 255, 255)`
- **THEN** the sign's style contains the ANSI escape for that color (`\x1b[38;2;85;255;255m`)

#### Scenario: Custom sign color override
- **WHEN** `init.lua` sets `y.theme.SignQfix = '#ff0000'`
- **THEN** the qfix sign renders with red foreground

### Requirement: Directory window borders are theme-configurable
The system SHALL apply `DirectoryBorderFg` and `DirectoryBorderBg` theme tokens to borders inside directory-type windows (parent, current, and preview panes).

#### Scenario: Custom directory window border colors
- **WHEN** `init.lua` sets `y.theme.DirectoryBorderFg = '#444444'` and `y.theme.DirectoryBorderBg = '#111111'`
- **THEN** the right-side borders between directory panes render with the configured foreground and background

#### Scenario: Default directory window border colors
- **WHEN** no `DirectoryBorderFg` or `DirectoryBorderBg` tokens are set
- **THEN** directory window borders render with black foreground and reset (transparent) background

### Requirement: Split borders are theme-configurable
The system SHALL rename the existing `BorderFg` token to `SplitBorderFg` and add a `SplitBorderBg` token. These tokens SHALL apply to vertical split separator borders. The `yeet-buffer` view SHALL use border colors from `BufferTheme` when rendering `Block` border widgets. No hardcoded `Color::Black` SHALL remain in the buffer border rendering code.

#### Scenario: Custom split border colors
- **WHEN** `init.lua` sets `y.theme.SplitBorderFg = '#555555'` and `y.theme.SplitBorderBg = '#000000'`
- **THEN** the vertical split separator renders with the configured foreground and background

#### Scenario: Default split border colors
- **WHEN** no `SplitBorderFg` or `SplitBorderBg` tokens are set
- **THEN** split borders render with black foreground and reset (transparent) background

#### Scenario: BorderFg token is removed
- **WHEN** `init.lua` references `y.theme.BorderFg`
- **THEN** the token does not exist; users must use `SplitBorderFg` instead

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
