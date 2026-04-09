## ADDED Requirements

### Requirement: Directory icon color tokens are theme-configurable
The system SHALL provide theme tokens for directory icon/text color classes introduced by the directory-icons plugin. These tokens SHALL be assignable from `init.lua` via `y.theme.<TokenName>` and participate in the same hex parsing and fallback behavior as existing tokens.

### Requirement: Default filename text color follows icon base color
When no override is configured, directory entry filename text SHALL use the same base color as the resolved icon's original default color.

#### Scenario: Rust entry defaults to rust orange for icon and text
- **WHEN** a file entry matches the `*.rs` mapping and no theme override is set
- **THEN** both the rust icon glyph and filename text render using the rust icon's default orange base color

#### Scenario: Class override affects icon and text together
- **WHEN** `init.lua` overrides a mapped class token with a valid hex color
- **THEN** both icon glyph and filename text for entries mapped to that class use the overridden color

### Requirement: Legacy directory-entry colorization is removed
The system SHALL remove the previous built-in file/directory colorization path for directory buffers before applying directory-icons-driven styling.

#### Scenario: Styling source is plugin-driven mapping only
- **WHEN** a directory buffer entry is rendered
- **THEN** foreground styling for icon/text is derived from directory-icons mapping and not from legacy built-in directory colorization rules

#### Scenario: Custom icon class color is applied
- **WHEN** `init.lua` sets a directory icon class token to a valid hex color
- **THEN** entries mapped to that class render icon and filename text with the configured foreground color

#### Scenario: Unset icon class token uses default
- **WHEN** no value is provided for an icon class token
- **THEN** the icon renderer uses the compiled-in default color for that token

### Requirement: Directory icon token fallback exists
The system SHALL expose a default/fallback directory icon color token used when an icon descriptor references an unknown or unmapped class.

#### Scenario: Unknown class falls back to default icon token
- **WHEN** a resolved icon descriptor references a class without an explicit token mapping
- **THEN** the renderer applies the fallback directory icon token color
