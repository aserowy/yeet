## ADDED Requirements

### Requirement: Directory icon resolution by file identity
The system SHALL resolve a directory-entry icon descriptor from the entry name and extension using the directory-icons plugin. For file entries with a recognized extension, the descriptor SHALL map to that extension's icon class. For unrecognized file entries, the descriptor SHALL fall back to a default file icon class.

### Requirement: Rule mapping applies to all matching entries
Icon/class/color resolution SHALL be rule-based by extension, exact filename, or directory name, and each rule SHALL apply uniformly to every matching entry.

### Requirement: One unified mapping configuration
The directory-icons plugin SHALL use a single, easy-to-extend mapping configuration that stores both file rules (extension/name) and directory-name rules.

#### Scenario: File and directory rules share one mapping source
- **WHEN** runtime loads directory-icons mapping configuration
- **THEN** file extension/name rules and directory-name rules are read from the same configuration structure

#### Scenario: New rule can be added without split config updates
- **WHEN** a user adds a new extension or directory-name mapping rule
- **THEN** only one mapping list/source needs to be updated for icon/class/color behavior to take effect

#### Scenario: Extension rule applies to all matching files
- **WHEN** multiple file entries in a directory buffer match the `*.rs` extension rule
- **THEN** each matching entry resolves to the rust icon/class mapping and associated default base color

#### Scenario: Named default directories use configured mapping
- **WHEN** a directory entry name is one of `.direnv`, `target`, `.git`, or `.github`
- **THEN** icon/class/color resolution uses the configured default directory-name mapping for that entry name

#### Scenario: Known Nerd Font file icon defaults are preseeded
- **WHEN** a file entry has a filename/extension with a corresponding Nerd Font icon in the default set
- **THEN** icon/class/color resolution uses the preseeded default mapping for that file entry

#### Scenario: Rust file resolves to rust icon class
- **WHEN** a directory buffer contains a file named `name.rs`
- **THEN** icon resolution returns the rust icon descriptor for that entry

#### Scenario: Unknown extension uses default file icon class
- **WHEN** a directory buffer contains a file named `README.unknownext`
- **THEN** icon resolution returns the default file icon descriptor

### Requirement: Directory icon descriptor includes theme token binding
The icon descriptor returned by the directory-icons plugin SHALL include the mapped style key(s) used to color that class for both icon glyph and filename text rendering.

#### Scenario: Descriptor carries token for color lookup
- **WHEN** icon resolution returns a descriptor for a recognized extension
- **THEN** the descriptor includes concrete style-token binding(s) that can be resolved by the theme system

### Requirement: Directory icon fallback safety
If icon resolution fails for any reason, the system SHALL render a stable fallback icon and fallback color token so directory rendering continues without error.

#### Scenario: Resolver failure degrades gracefully
- **WHEN** icon lookup for an entry raises an error in plugin code
- **THEN** the entry renders with the configured fallback icon and fallback icon color token
