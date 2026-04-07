## ADDED Requirements

### Requirement: pluginlist command

The system SHALL provide a `:pluginlist` command that synchronously prints the list of registered plugins with their status. For each plugin, the output SHALL include the plugin URL, resolved version/commit, and status (`loaded`, `error`, `missing`). For plugins with `error` or `missing` status, the error message SHALL be included.

#### Scenario: All plugins loaded

- **WHEN** the user executes `:pluginlist` and all registered plugins are loaded
- **THEN** the system SHALL print each plugin's URL and status as `loaded`

#### Scenario: Some plugins with errors

- **WHEN** the user executes `:pluginlist` and plugin A failed to load with error "attempt to call nil"
- **THEN** the output SHALL show plugin A with status `error` and the message "attempt to call nil"

#### Scenario: Missing plugins

- **WHEN** the user executes `:pluginlist` and plugin B is missing from disk
- **THEN** the output SHALL show plugin B with status `missing`

#### Scenario: No plugins registered

- **WHEN** the user executes `:pluginlist` and no plugins are registered
- **THEN** the system SHALL print a message indicating no plugins are registered

#### Scenario: Dependencies shown

- **WHEN** the user executes `:pluginlist` and plugin A has a dependency on library B
- **THEN** both plugin A and library B SHALL appear in the output

### Requirement: pluginsync command

The system SHALL provide a `:pluginsync` command that dispatches an async task to restore all registered plugins to their locked versions. Progress and results SHALL be shown in the tasks window.

#### Scenario: Sync succeeds

- **WHEN** the user executes `:pluginsync` and all plugins sync successfully
- **THEN** the system SHALL print a success message in the command line with the number of plugins synced

#### Scenario: Sync with missing lock file

- **WHEN** the user executes `:pluginsync` and no lock file exists
- **THEN** the system SHALL print an error in the command line suggesting to run `:pluginupdate` first

#### Scenario: Sync with integrity error

- **WHEN** the user executes `:pluginsync` and a plugin's tree SHA-256 does not match the lock file
- **THEN** the system SHALL print an integrity error for that plugin in the command line

### Requirement: pluginupdate command

The system SHALL provide a `:pluginupdate` command that dispatches an async task to resolve and fetch the latest allowed versions for all registered plugins. The lock file SHALL be updated with new commit SHAs and tree hashes. Progress and results SHALL be shown in the tasks window.

#### Scenario: Update succeeds

- **WHEN** the user executes `:pluginupdate` and all plugins update successfully
- **THEN** the system SHALL print a success message in the command line with the number of plugins updated and the lock file SHALL be written

#### Scenario: Update with version resolution failure

- **WHEN** the user executes `:pluginupdate` and plugin A has no matching remote tag for its version constraint
- **THEN** the system SHALL print an error for plugin A in the command line while other plugins are still updated

#### Scenario: First update creates lock file

- **WHEN** the user executes `:pluginupdate` and no lock file exists
- **THEN** the system SHALL create the lock file with entries for all resolved plugins

### Requirement: Sync and update clean up unregistered plugins

The `:pluginsync` and `:pluginupdate` commands SHALL remove plugins from the lock file and data directory that are no longer registered in `init.lua`.

#### Scenario: Cleanup reported in output

- **WHEN** `:pluginupdate` removes two unregistered plugins
- **THEN** the system SHALL print a message in the command line listing the removed plugins
