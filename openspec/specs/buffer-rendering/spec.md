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
