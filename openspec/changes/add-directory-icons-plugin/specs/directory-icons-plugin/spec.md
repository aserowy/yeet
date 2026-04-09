## ADDED Requirements

### Requirement: Plugin owns all icon identification and text color logic
The `yeet-directory-icons` plugin SHALL contain all logic for determining which icon glyph to display and how to color both the icon glyph and the filename text. The core SHALL NOT contain any icon resolution tables, extension mappings, or color rules; it only invokes hooks and the plugin directly mutates bufferlines.

### Requirement: Plugin directly mutates bufferlines via hooks in EnumerationChanged/EnumerationFinished/PathsAdded handling
The plugin SHALL implement hook handlers that are invoked during the existing `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling. Each hook call receives the complete bufferline and the given window with all metadata. The plugin **directly mutates the bufferline**: it adds or replaces the icon in the icon column and colors the bufferline text. There is no request/response pattern — the plugin edits the bufferline in-place inside the hook handler.

#### Scenario: Plugin receives full bufferline and window context during enumeration
- **WHEN** the core processes an `EnumerationChanged` or `EnumerationFinished` message and invokes the hook for a bufferline
- **THEN** the hook call provides the complete bufferline data and the given window with all metadata to the plugin

#### Scenario: Plugin receives full bufferline and window context during path addition
- **WHEN** the core processes a `PathsAdded` message and invokes the hook for a new bufferline
- **THEN** the hook call provides the complete bufferline data and the given window with all metadata to the plugin

#### Scenario: Plugin sets icon glyph for a recognized file
- **WHEN** the plugin's hook handler receives a bufferline for a file with extension `.rs`
- **THEN** the plugin directly sets the rust icon glyph in the icon column and applies rust color to both icon and filename text on the bufferline

#### Scenario: Plugin sets fallback icon for unrecognized file
- **WHEN** the plugin's hook handler receives a bufferline for a file named `README.unknownext`
- **THEN** the plugin directly sets the default file icon glyph and applies default color on the bufferline

#### Scenario: Plugin sets directory-specific icon
- **WHEN** the plugin's hook handler receives a bufferline for a directory entry named `.git`
- **THEN** the plugin directly sets the git directory icon glyph and applies directory color on the bufferline (using a directory-specific token distinct from the file default)

#### Scenario: Plugin replaces existing icon on re-processing
- **WHEN** a bufferline already has an icon set and the hook is invoked again (e.g., during `EnumerationFinished` after `EnumerationChanged`)
- **THEN** the plugin replaces the existing icon with the newly resolved icon

### Requirement: Rule mapping applies to all matching entries
Icon/class/color resolution SHALL be rule-based by extension, exact filename, or directory name, and each rule SHALL apply uniformly to every matching entry. All rule logic lives in the plugin.

### Requirement: One unified mapping configuration in the plugin
The directory-icons plugin SHALL use a single, easy-to-extend mapping configuration that stores both file rules (extension/name) and directory-name rules.

#### Scenario: File and directory rules share one mapping source
- **WHEN** the plugin loads its mapping configuration
- **THEN** file extension/name rules and directory-name rules are read from the same configuration structure

#### Scenario: New rule can be added without split config updates
- **WHEN** a user adds a new extension or directory-name mapping rule
- **THEN** only one mapping list/source needs to be updated for icon/class/color behavior to take effect

#### Scenario: Extension rule applies to all matching files
- **WHEN** the plugin's hook handler processes multiple file entries matching the `*.rs` extension rule
- **THEN** each matching entry's bufferline is mutated to the rust icon/class mapping and associated default base color

#### Scenario: Named default directories use configured mapping
- **WHEN** a directory entry name is one of `.direnv`, `target`, `.git`, or `.github`
- **THEN** the plugin's icon/class/color mutation uses the configured default directory-name mapping for that entry name

#### Scenario: Known Nerd Font file icon defaults are preseeded
- **WHEN** a file entry has a filename/extension with a corresponding Nerd Font icon in the default set
- **THEN** the plugin's icon/class/color mutation uses the preseeded default mapping for that file entry

### Requirement: Plugin-defined token names
The plugin SHALL define its own token names for icon/text color classes. The core does not standardize icon-color token class names. Directories SHALL use a distinct icon token separate from the file default token.

#### Scenario: Directory entries use a separate token from file entries
- **WHEN** the plugin mutates a directory entry's bufferline
- **THEN** the applied color token is a directory-specific token, distinct from the default file icon token

### Requirement: Plugin fallback safety
If the plugin's hook handler fails for any reason, the core SHALL preserve the bufferline in its pre-hook state (empty icon column, default text color) so directory rendering continues without error.

#### Scenario: Plugin hook failure degrades gracefully
- **WHEN** a hook call to the plugin raises an error
- **THEN** the bufferline retains its pre-hook state with no icon and default text styling
