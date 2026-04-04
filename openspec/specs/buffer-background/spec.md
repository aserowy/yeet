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
The system SHALL ensure that every line in the buffer — including non-cursor lines — renders with `BufferBg` as its background. ANSI reset sequences (`\x1b[0m`) used in line numbers, signs, search highlights, and cursor line styling SHALL re-apply the `BufferBg` background so that resets do not create "holes" that fall back to the terminal default.

#### Scenario: Non-cursor line has BufferBg background
- **WHEN** `BufferBg` is set to `#1e1e2e` and a line is not the cursor line
- **THEN** the line content renders with `#1e1e2e` background, not the terminal default

#### Scenario: ANSI reset in line number does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a relative line number is rendered with an `\x1b[0m` reset
- **THEN** the background after the reset is `BufferBg`, not the terminal default

#### Scenario: ANSI reset in search highlight does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a search match highlight ends with a reset
- **THEN** the background after the search highlight reset is `BufferBg`, not the terminal default

#### Scenario: ANSI reset in sign does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a sign is rendered with an `\x1b[0m` reset
- **THEN** the background after the sign reset is `BufferBg`, not the terminal default
