## MODIFIED Requirements

### Requirement: Color token names cover all UI elements
The system SHALL define color tokens for at minimum the following UI elements:
- Tabbar: active tab foreground/background, inactive tab foreground/background, tabbar background
- Statusline: foreground, background (focused and unfocused variants), diff added/modified/removed colors, permissions foreground, border foreground, border background
- Commandline: foreground, background
- Buffer: cursor line background, search highlight background, line number foreground, current line number foreground, sign column foreground, file entry foreground, directory entry foreground
- Directory window: border foreground, border background
- Split: border foreground, border background
- Cursor: normal mode style, insert mode style
- Signs: quickfix sign color, mark sign color

#### Scenario: All hardcoded colors have corresponding tokens
- **WHEN** the default theme is loaded with no user overrides
- **THEN** the rendered UI is visually identical to the current hardcoded appearance
