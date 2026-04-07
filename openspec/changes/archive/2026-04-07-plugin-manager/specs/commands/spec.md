## ADDED Requirements

### Requirement: pluginlist command is recognized

The system SHALL recognize `:pluginlist` as a valid command in the command dispatch. It SHALL be executed synchronously and print output to the command line area, following the same pattern as `:marks` and `:junk`.

#### Scenario: pluginlist dispatched

- **WHEN** the user types `:pluginlist` and presses enter
- **THEN** the system SHALL execute the pluginlist handler and display the output

#### Scenario: pluginlist with arguments

- **WHEN** the user types `:pluginlist somearg`
- **THEN** the system SHALL ignore the argument and display the full plugin list

### Requirement: pluginsync command is recognized

The system SHALL recognize `:pluginsync` as a valid command in the command dispatch. It SHALL dispatch an async task and return the user to the previous mode.

#### Scenario: pluginsync dispatched

- **WHEN** the user types `:pluginsync` and presses enter
- **THEN** the system SHALL create an async task for the sync operation

### Requirement: pluginupdate command is recognized

The system SHALL recognize `:pluginupdate` as a valid command in the command dispatch. It SHALL dispatch an async task and return the user to the previous mode.

#### Scenario: pluginupdate dispatched

- **WHEN** the user types `:pluginupdate` and presses enter
- **THEN** the system SHALL create an async task for the update operation
