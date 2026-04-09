## ADDED Requirements

### Requirement: Directory icon color tokens are theme-configurable
The system SHALL provide theme tokens for directory icon/text color classes. These tokens SHALL be assignable from `init.lua` via `y.theme.<TokenName>` and participate in the same hex parsing and fallback behavior as existing tokens. Token names are plugin-defined — the core does not standardize icon-color class names.

### Requirement: Default filename text color follows icon base color
When no override is configured, directory entry filename text SHALL use the same base color as the resolved icon's original default color. This color is applied by the plugin directly mutating the bufferline during mutation hooks in `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling.

#### Scenario: Rust entry defaults to rust orange for icon and text
- **WHEN** a file entry matches the `*.rs` mapping and no theme override is set
- **THEN** the plugin directly sets rust default color on the bufferline, and both the rust icon glyph and filename text render using that color

#### Scenario: Class override affects icon and text together
- **WHEN** `init.lua` overrides a mapped class token with a valid hex color
- **THEN** both icon glyph and filename text for entries mapped to that class use the overridden color

### Requirement: Directories use a distinct icon token from file default
Directory entries SHALL have their own separate icon color token, distinct from the file default icon token, allowing independent visual treatment.

#### Scenario: Directory token is separate from file default
- **WHEN** a directory entry is rendered with an icon
- **THEN** the applied token is a directory-specific token, not the same as the file default token

### Requirement: Legacy directory-entry colorization is removed
The system SHALL remove the previous built-in file/directory colorization path for directory buffers before applying plugin-driven styling via mutation hooks in `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling.

#### Scenario: Styling source is plugin mutation only
- **WHEN** a directory buffer entry is rendered
- **THEN** foreground styling for icon/text is derived from plugin bufferline mutations and not from legacy built-in directory colorization rules

#### Scenario: Custom icon class color is applied
- **WHEN** `init.lua` sets a directory icon class token to a valid hex color
- **THEN** entries mapped to that class render icon and filename text with the configured foreground color

#### Scenario: Unset icon class token uses default
- **WHEN** no value is provided for an icon class token
- **THEN** the renderer uses the compiled-in default color for that token (as set by the plugin)

### Requirement: Directory icon token fallback exists
The system SHALL expose a default/fallback directory icon color token used when an unmapped class is encountered.

#### Scenario: Unknown class falls back to default icon token
- **WHEN** a bufferline's color references a class without an explicit token mapping
- **THEN** the renderer applies the fallback directory icon token color
