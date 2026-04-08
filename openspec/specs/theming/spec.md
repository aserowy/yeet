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

### Requirement: BufferTheme has no default construction
The `yeet_buffer::BufferTheme` struct SHALL NOT implement `Default`. Callers MUST provide all field values explicitly when constructing a `BufferTheme`. All fields SHALL be of type `ratatui::style::Color`. The struct SHALL NOT contain ANSI escape code strings, reset sequences, or cursor mode codes.

#### Scenario: Compile-time enforcement
- **WHEN** a consumer attempts `BufferTheme::default()`
- **THEN** the code fails to compile with a missing trait implementation error

#### Scenario: All fields are Color type
- **WHEN** inspecting the `BufferTheme` struct definition
- **THEN** every field is of type `ratatui::style::Color` with no `String` fields

#### Scenario: No cursor mode or reset fields
- **WHEN** inspecting the `BufferTheme` struct definition
- **THEN** there are no fields for cursor normal/insert codes, cursor line reset, or any ANSI escape sequences

### Requirement: Single view entry point requires a theme
The `yeet_buffer` crate SHALL expose a single public `view()` function whose signature includes a `&BufferTheme` parameter. There SHALL NOT be a convenience variant that supplies a default theme internally.

#### Scenario: Buffer view with explicit theme
- **WHEN** `yeet_buffer::view(viewport, mode, buffer, theme, frame)` is called with a valid `BufferTheme`
- **THEN** the buffer is rendered using the colors from the provided theme

#### Scenario: No implicit-default view function exists
- **WHEN** a consumer attempts to call a `view()` function without a `&BufferTheme` parameter
- **THEN** the code fails to compile because no such function signature exists

### Requirement: Frontend theme is the sole BufferTheme factory
The `yeet_frontend::theme::Theme` SHALL expose only `to_buffer_theme_with_border(fg_token, bg_token)` as the construction method for `BufferTheme`. The convenience method `to_buffer_theme()` SHALL NOT exist. All call sites MUST explicitly specify which border fg/bg tokens to use.

#### Scenario: All BufferTheme fields derived from theme tokens
- **WHEN** `Theme::to_buffer_theme_with_border(fg, bg)` is called
- **THEN** the returned `BufferTheme` contains `Color` values resolved from the token map, with `border_fg` and `border_bg` from the specified tokens

#### Scenario: BufferTheme fields map to tokens
- **WHEN** `Theme::to_buffer_theme_with_border(SPLIT_BORDER_FG, SPLIT_BORDER_BG)` is called with default theme
- **THEN** `cursor_line_bg` equals `Color::Rgb(128, 128, 128)`, `search_bg` equals `Color::Red`, `line_nr` equals `Color::Rgb(128, 128, 128)`, `cur_line_nr` equals `Color::White`, `border_fg` equals `Color::Black`, `border_bg` equals `Color::Reset`

#### Scenario: No convenience method with implicit border tokens
- **WHEN** a consumer attempts to call `theme.to_buffer_theme()` without border token arguments
- **THEN** the code fails to compile because no such method exists

#### Scenario: Directory pane uses directory border tokens
- **WHEN** a directory pane constructs its `BufferTheme`
- **THEN** it calls `to_buffer_theme_with_border(DIRECTORY_BORDER_FG, DIRECTORY_BORDER_BG)`

#### Scenario: Split pane uses split border tokens
- **WHEN** a split pane constructs its `BufferTheme`
- **THEN** it calls `to_buffer_theme_with_border(SPLIT_BORDER_FG, SPLIT_BORDER_BG)`

#### Scenario: Directory window inside a vertical split uses split border for separator
- **WHEN** a directory window is the first child of a vertical split
- **THEN** the preview pane's border (the split separator) uses `SPLIT_BORDER_FG`/`SPLIT_BORDER_BG`, not `DIRECTORY_BORDER_FG`/`DIRECTORY_BORDER_BG`

#### Scenario: Directory panes do not inherit draw_borders from split context
- **WHEN** a directory window is inside a vertical split with `draw_borders: Some(true)`
- **THEN** the parent and current panes use their own viewport `show_border` setting, not the split's `draw_borders` override

### Requirement: Buffer background color is theme-configurable
The system SHALL apply the `BufferBg` theme token as the background color for the buffer content area. The `BufferTheme` struct SHALL include a `buffer_bg` field. The buffer rendering layer SHALL use this color as the base background when drawing the content area.

#### Scenario: Custom buffer background
- **WHEN** `init.lua` sets `y.theme.BufferBg = '#1e1e2e'`
- **THEN** the buffer content area renders with the configured dark background

#### Scenario: Default buffer background preserves terminal default
- **WHEN** no `BufferBg` token is set
- **THEN** the buffer content area renders with `Color::Reset` (terminal default background)

#### Scenario: BufferBg is passed through BufferTheme
- **WHEN** the frontend converts the theme via `to_buffer_theme_with_border()`
- **THEN** the resulting `BufferTheme` contains the `buffer_bg` value from the `BufferBg` token

### Requirement: Buffer line content respects BufferBg after ANSI resets
The system SHALL ensure that every line in the buffer renders with the correct background after ANSI resets. On non-cursor lines, reset sequences SHALL re-apply `BufferBg`. On the cursor line (when cursor line highlighting is active), reset sequences SHALL re-apply `cursor_line_bg` instead of `BufferBg`, so that the cursor line background is continuous across the entire line including after search highlights.

#### Scenario: Non-cursor line has BufferBg background
- **WHEN** `BufferBg` is set to `#1e1e2e` and a line is not the cursor line
- **THEN** the line content renders with `#1e1e2e` background, not the terminal default

#### Scenario: ANSI reset in line number does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a relative line number is rendered with an `\x1b[0m` reset
- **THEN** the background after the reset is `BufferBg`, not the terminal default

#### Scenario: ANSI reset in search highlight does not clear BufferBg on non-cursor line
- **WHEN** `BufferBg` is set to a custom color, a search match highlight ends with a reset, and the line is not the cursor line
- **THEN** the background after the search highlight reset is `BufferBg`, not the terminal default

#### Scenario: ANSI reset in search highlight uses cursor_line_bg on cursor line
- **WHEN** a search match appears on the cursor line and the search highlight ends with a reset
- **THEN** the background after the search highlight reset is `cursor_line_bg`, not `BufferBg`

#### Scenario: Multiple search matches on cursor line preserve cursor_line_bg between matches
- **WHEN** the cursor line contains two or more search matches
- **THEN** the background between and after each match is `cursor_line_bg`

#### Scenario: Search highlight on cursor line with hide_cursor_line uses BufferBg
- **WHEN** a search match appears on a line that would be the cursor line, but `hide_cursor_line` is true
- **THEN** the background after the search highlight reset is `BufferBg`, not `cursor_line_bg`

#### Scenario: ANSI reset in sign does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a sign is rendered with an `\x1b[0m` reset
- **THEN** the background after the sign reset is `BufferBg`, not the terminal default

### Requirement: BufferTheme is created at point of use
Functions that call `yeet_buffer::view()` SHALL create the `BufferTheme` locally via `theme.to_buffer_theme()` or `theme.to_buffer_theme_with_border()` rather than receiving it as a threaded parameter from parent functions. The constructed `BufferTheme` SHALL contain only `Color` values.

#### Scenario: Intermediate layout functions do not accept BufferTheme
- **WHEN** `render_window` is called to lay out split or directory windows
- **THEN** its signature does not include a `BufferTheme` parameter

#### Scenario: Leaf rendering functions create BufferTheme locally
- **WHEN** `render_buffer_slot` or `render_directory_buffer` needs to call `yeet_buffer::view()`
- **THEN** it creates a `BufferTheme` from the `&Theme` parameter and passes it to the view call

### Requirement: from_enumeration accepts Theme reference
The `from_enumeration` function SHALL accept a `&Theme` parameter instead of individual ANSI color strings. It SHALL extract the needed color tokens (`BufferFileFg`, `BufferDirectoryFg`) internally from the theme.

#### Scenario: Directory entry uses theme token
- **WHEN** `from_enumeration` is called with `ContentKind::Directory` and a theme where `BufferDirectoryFg` is `Color::LightBlue`
- **THEN** the returned `BufferLine` content contains the ANSI code for light blue foreground (`\x1b[94m`)

#### Scenario: File entry uses theme token
- **WHEN** `from_enumeration` is called with `ContentKind::File` and a theme where `BufferFileFg` is `Color::White`
- **THEN** the returned `BufferLine` content contains the ANSI code for white foreground (`\x1b[37m`)

#### Scenario: Call sites pass only content, kind, and theme
- **WHEN** callers invoke `from_enumeration`
- **THEN** they pass `(content, kind, theme)` with no pre-computed ANSI strings

### Requirement: Cursor mode codes are buffer-view constants
The buffer view module SHALL define cursor mode codes (`\x1b[7m`, `\x1b[27m`, `\x1b[4m`, `\x1b[24m`) and reset code (`\x1b[0m`) as module-level constants. These SHALL NOT be part of `BufferTheme`.

#### Scenario: Cursor normal mode uses constant
- **WHEN** the buffer view renders a cursor in Normal mode
- **THEN** it uses a module-level constant for the reverse-video code, not a `BufferTheme` field

#### Scenario: Cursor line reset uses constant
- **WHEN** the buffer view renders the cursor line background
- **THEN** it uses a module-level constant for the ANSI reset code, not a `BufferTheme` field

### Requirement: Semantic status color tokens

The theme SHALL include `ErrorFg`, `WarningFg`, `SuccessFg`, and `InformationFg` color tokens for semantic coloring. These tokens SHALL be configurable via `y.theme.ErrorFg`, `y.theme.WarningFg`, `y.theme.SuccessFg`, and `y.theme.InformationFg` in `init.lua`.

#### Scenario: Default error color

- **WHEN** the user has not configured `y.theme.ErrorFg`
- **THEN** the system SHALL use red as the default error foreground color

#### Scenario: Default warning color

- **WHEN** the user has not configured `y.theme.WarningFg`
- **THEN** the system SHALL use yellow as the default warning foreground color

#### Scenario: Default success color

- **WHEN** the user has not configured `y.theme.SuccessFg`
- **THEN** the system SHALL use green as the default success foreground color

#### Scenario: Default information color

- **WHEN** the user has not configured `y.theme.InformationFg`
- **THEN** the system SHALL use blue as the default information foreground color

#### Scenario: Custom error color

- **WHEN** `init.lua` contains `y.theme.ErrorFg = '#ff0000'`
- **THEN** the system SHALL use `#ff0000` as the error foreground color in all error-colored output
