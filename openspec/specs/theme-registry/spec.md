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
- Statusline: foreground, background (focused and unfocused variants), diff added/modified/removed colors
- Commandline: foreground, background
- Buffer: cursor line background, search highlight background, line number foreground, current line number foreground, sign column foreground
- Cursor: normal mode style, insert mode style
- Signs: quickfix sign color, mark sign color

#### Scenario: All hardcoded colors have corresponding tokens
- **WHEN** the default theme is loaded with no user overrides
- **THEN** the rendered UI is visually identical to the current hardcoded appearance

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
