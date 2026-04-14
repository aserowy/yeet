## ADDED Requirements

### Requirement: Directory icon color tokens follow DirectoryIconsColor naming convention
The plugin SHALL register theme tokens using the `DirectoryIconsColor*` naming convention (e.g., `DirectoryIconsColorRs`, `DirectoryIconsColorTxt`, `DirectoryIconsColorMakefile`, `DirectoryIconsColorDotEnv`, `DirectoryIconsColorGoMod`, `DirectoryIconsColorDefaultDirectory`, `DirectoryIconsColorDefaultFile`). These tokens SHALL be assignable from `init.lua` via `y.theme.<TokenName>` and participate in the same hex parsing and fallback behavior as existing tokens. Token names are plugin-defined — the core does not standardize icon-color class names. Documentation of these tokens SHALL live in the plugin's own `docs/help/` directory, not in core documentation.

### Requirement: Default filename text color follows icon base color
When no override is configured, directory entry filename text SHALL use the same base color as the resolved icon's original default color. This color is applied by the plugin directly mutating the bufferline content Ansi string during mutation hooks.

#### Scenario: Rust entry defaults to rust orange for icon and text
- **WHEN** a file entry matches the `*.rs` mapping and no theme override is set for `DirectoryIconsColorRs`
- **THEN** the plugin reads `y.theme.DirectoryIconsColorRs` (set to the default during `setup()`) and uses that color for both icon glyph and filename text

#### Scenario: Class override affects icon and text together
- **WHEN** `init.lua` overrides `DirectoryIconsColorRs` with a valid hex color
- **THEN** both icon glyph and filename text for `*.rs` entries use the overridden color

### Requirement: Directories use distinct DirectoryIconsColor token from file default
Directory entries SHALL have their own separate icon color token (`DirectoryIconsColorDefaultDirectory`), distinct from the file default icon token (`DirectoryIconsColorDefaultFile`), allowing independent visual treatment.

#### Scenario: Directory token is separate from file default
- **WHEN** a directory entry is rendered with an icon
- **THEN** the applied token is a directory-specific token, not the same as the file default token

### Requirement: Theme plugins can override DirectoryIconsColor tokens
Theme plugins (e.g., `yeet-bluloco-theme`) SHALL be able to set `DirectoryIconsColor*` theme tokens. When a theme plugin sets a token value before the directory-icons plugin runs `setup()`, the directory-icons plugin SHALL respect the theme-provided value and not overwrite it. The `yeet-bluloco-theme` plugin SHALL provide override values for all `DirectoryIconsColor*` tokens.

#### Scenario: Theme plugin overrides DirectoryIconsColorDefaultDirectory
- **WHEN** `yeet-bluloco-theme` sets `DirectoryIconsColorDefaultDirectory` to a custom color before `yeet-directory-icons.setup()` runs
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
- **WHEN** `init.lua` sets `DirectoryIconsColorRs` to a valid hex color
- **THEN** entries matching the `*.rs` rule render icon and filename text with the configured foreground color

#### Scenario: Unset icon class token uses plugin default
- **WHEN** no value is provided for an icon class token and no theme plugin overrides it
- **THEN** the plugin uses the compiled-in Nerd Font default color for that token

### Requirement: Directory icon token fallback exists
The plugin SHALL expose `DirectoryIconsColorDefaultFile` and `DirectoryIconsColorDefaultDirectory` as fallback color tokens used when an entry has no explicit icon mapping.

#### Scenario: Unknown class falls back to default icon token
- **WHEN** a bufferline's entry has no explicit icon mapping
- **THEN** the plugin applies the fallback default icon token color
