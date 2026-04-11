## ADDED Requirements

### Requirement: Directory icon color tokens are theme-configurable
The system SHALL provide theme tokens for directory icon/text color classes. These tokens SHALL be assignable from `init.lua` via `y.theme.<TokenName>` and participate in the same hex parsing and fallback behavior as existing tokens. Token names are plugin-defined — the core does not standardize icon-color class names.

### Requirement: Default filename text color follows icon base color
When no override is configured, directory entry filename text SHALL use the same base color as the resolved icon's original default color. This color is applied by the plugin directly mutating the bufferline content Ansi string during mutation hooks.

#### Scenario: Rust entry defaults to rust orange for icon and text
- **WHEN** a file entry matches the `*.rs` mapping and no theme override is set
- **THEN** the plugin directly prepends rust default color ANSI sequence to the content string, and both the rust icon glyph and filename text render using that color

#### Scenario: Class override affects icon and text together
- **WHEN** `init.lua` overrides a mapped class token with a valid hex color
- **THEN** both icon glyph and filename text for entries mapped to that class use the overridden color

### Requirement: Directories use a distinct icon token from file default
Directory entries SHALL have their own separate icon color token, distinct from the file default icon token, allowing independent visual treatment.

#### Scenario: Directory token is separate from file default
- **WHEN** a directory entry is rendered with an icon
- **THEN** the applied token is a directory-specific token, not the same as the file default token

### Requirement: Theme plugins can override icon tokens
Theme plugins (e.g., `yeet-bluloco-theme`) SHALL be able to set icon-related theme tokens. When a theme plugin sets a token value before the directory-icons plugin processes entries, the directory-icons plugin SHALL respect the theme-provided value and not overwrite it.

#### Scenario: Theme plugin overrides BufferDirectoryFg
- **WHEN** `yeet-bluloco-theme` sets `BufferDirectoryFg` to a custom color
- **THEN** the directory-icons plugin uses the theme-provided color for directory entries instead of its own default

#### Scenario: Theme plugin does not set icon token
- **WHEN** no theme plugin has set a value for a specific icon token
- **THEN** the directory-icons plugin uses its built-in default color

### Requirement: Legacy directory-entry colorization is removed without fallback
The system SHALL remove the previous built-in file/directory colorization path for directory buffers. There is no fallback — without the plugin, directory entries render as plain unstyled text.

#### Scenario: Styling source is plugin mutation only
- **WHEN** a directory buffer entry is rendered
- **THEN** foreground styling for icon/text is derived from plugin bufferline mutations and not from any core colorization logic

#### Scenario: No plugin means no color
- **WHEN** `yeet-directory-icons` is not installed
- **THEN** directory entries render as plain unstyled text with no foreground color

#### Scenario: Custom icon class color is applied
- **WHEN** `init.lua` sets a directory icon class token to a valid hex color
- **THEN** entries mapped to that class render icon and filename text with the configured foreground color

#### Scenario: Unset icon class token uses plugin default
- **WHEN** no value is provided for an icon class token and no theme plugin overrides it
- **THEN** the plugin uses the compiled-in Nerd Font default color for that token

### Requirement: Directory icon token fallback exists
The plugin SHALL expose a default/fallback directory icon color token used when an unmapped class is encountered.

#### Scenario: Unknown class falls back to default icon token
- **WHEN** a bufferline's entry has no explicit icon mapping
- **THEN** the plugin applies the fallback default icon token color
