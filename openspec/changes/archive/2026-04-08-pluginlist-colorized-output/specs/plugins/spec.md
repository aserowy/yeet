## MODIFIED Requirements

### Requirement: pluginlist command

The system SHALL provide a `:pluginlist` command that synchronously prints the list of registered plugins with their status. For each plugin, the output SHALL include the plugin URL, resolved version/commit, and status (`loaded`, `error`, `missing`). For plugins with `error` or `missing` status, the error message SHALL be included. Each line SHALL be colored by status: loaded in success color, missing in warning color, error in error color.

#### Scenario: Loaded plugin shown in success color

- **WHEN** the user executes `:pluginlist` and plugin A is loaded
- **THEN** plugin A's line SHALL be rendered in the `SuccessFg` theme color

#### Scenario: Missing plugin shown in warning color

- **WHEN** the user executes `:pluginlist` and plugin B is missing from disk
- **THEN** plugin B's line SHALL be rendered in the `WarningFg` theme color

#### Scenario: Error plugin shown in error color

- **WHEN** the user executes `:pluginlist` and plugin C failed to load
- **THEN** plugin C's line SHALL be rendered in the `ErrorFg` theme color
