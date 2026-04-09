## ADDED Requirements

### Requirement: Directory icon resolution by file identity
The system SHALL resolve a directory-entry icon descriptor from the entry name and extension using the directory-icons plugin. For file entries with a recognized extension, the descriptor SHALL map to that extension's icon class. For unrecognized file entries, the descriptor SHALL fall back to a default file icon class.

#### Scenario: Rust file resolves to rust icon class
- **WHEN** a directory buffer contains a file named `name.rs`
- **THEN** icon resolution returns the rust icon descriptor for that entry

#### Scenario: Unknown extension uses default file icon class
- **WHEN** a directory buffer contains a file named `README.unknownext`
- **THEN** icon resolution returns the default file icon descriptor

### Requirement: Directory icon descriptor includes theme token binding
The icon descriptor returned by the directory-icons plugin SHALL include the theme token key used to color that icon class.

#### Scenario: Descriptor carries token for color lookup
- **WHEN** icon resolution returns a descriptor for a recognized extension
- **THEN** the descriptor includes a concrete token name that can be resolved by the theme system

### Requirement: Directory icon fallback safety
If icon resolution fails for any reason, the system SHALL render a stable fallback icon and fallback color token so directory rendering continues without error.

#### Scenario: Resolver failure degrades gracefully
- **WHEN** icon lookup for an entry raises an error in plugin code
- **THEN** the entry renders with the configured fallback icon and fallback icon color token
