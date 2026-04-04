## Purpose

Documentation completeness - ensures the README accurately reflects all implemented features.

## Requirements

### Requirement: README documents all implemented commands and keybindings
The README.md SHALL list every implemented command and keybinding in its shortcuts and commands tables.

#### Scenario: copen command is documented
- **WHEN** a user reads the README commands table
- **THEN** the `:copen` command SHALL be listed with its description

#### Scenario: gg and G keybindings are documented
- **WHEN** a user reads the navigation and normal mode keybindings table
- **THEN** `gg` and `G` SHALL be listed for jumping to top/bottom

#### Scenario: Enter keybinding is documented
- **WHEN** a user reads the navigation mode keybindings table
- **THEN** `Enter` SHALL be listed for opening the selected entry
