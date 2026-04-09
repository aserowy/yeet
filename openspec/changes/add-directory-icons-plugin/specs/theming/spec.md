## ADDED Requirements

### Requirement: Directory icon color tokens are theme-configurable
The system SHALL provide theme tokens for directory-icon color classes introduced by the directory-icons plugin. These tokens SHALL be assignable from `init.lua` via `y.theme.<TokenName>` and participate in the same hex parsing and fallback behavior as existing tokens.

#### Scenario: Custom icon class color is applied
- **WHEN** `init.lua` sets a directory icon class token to a valid hex color
- **THEN** icons mapped to that class render with the configured foreground color

#### Scenario: Unset icon class token uses default
- **WHEN** no value is provided for an icon class token
- **THEN** the icon renderer uses the compiled-in default color for that token

### Requirement: Directory icon token fallback exists
The system SHALL expose a default/fallback directory icon color token used when an icon descriptor references an unknown or unmapped class.

#### Scenario: Unknown class falls back to default icon token
- **WHEN** a resolved icon descriptor references a class without an explicit token mapping
- **THEN** the renderer applies the fallback directory icon token color
