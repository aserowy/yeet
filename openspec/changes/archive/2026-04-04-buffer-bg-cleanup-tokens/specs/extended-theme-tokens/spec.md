## ADDED Requirements

### Requirement: BufferBg token exists in the token set
The system SHALL define a `BUFFER_BG` constant in the `tokens` module with the string value `"BufferBg"`. The `Theme::default()` implementation SHALL insert a default value of `Color::Reset` for this token.

#### Scenario: BufferBg token default
- **WHEN** a default theme is created
- **THEN** `theme.color(tokens::BUFFER_BG)` returns `Color::Reset`

#### Scenario: BufferBg token override
- **WHEN** `init.lua` sets `y.theme.BufferBg = '#282828'`
- **THEN** `theme.color(tokens::BUFFER_BG)` returns `Color::Rgb(40, 40, 40)`

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

## REMOVED Requirements

### Requirement: CommandLineFg and CommandLineBg tokens
**Reason**: These tokens are defined but never consumed by any rendering code. No commandline component reads them.
**Migration**: No migration needed — setting these tokens never had a visual effect. They will be silently ignored if present in config.
