## MODIFIED Requirements

### Requirement: Plugin registration via y.plugin.register()

#### Scenario: Register with non-HTTPS URL is rejected

- **WHEN** `init.lua` contains `y.plugin.register({ url = "git@github.com:user/repo.git" })`
- **THEN** the system SHALL log an error indicating only HTTPS URLs are supported and SHALL NOT add the entry to the plugin list

#### Scenario: Register with HTTP URL is rejected

- **WHEN** `init.lua` contains `y.plugin.register({ url = "http://github.com/user/repo" })`
- **THEN** the system SHALL log an error indicating only HTTPS URLs are supported and SHALL NOT add the entry to the plugin list

#### Scenario: Register with HTTPS URL succeeds

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/repo" })`
- **THEN** the plugin specification SHALL be stored in memory

## ADDED Requirements

### Requirement: Git operations do not prompt for credentials

The system SHALL configure git operations to never prompt for credentials on stdin. Authentication failures SHALL produce clean error messages per-plugin without blocking the terminal or breaking application state.

#### Scenario: Private HTTPS repo fails cleanly

- **WHEN** a plugin URL points to a private HTTPS repository and `:pluginupdate` is executed
- **THEN** the system SHALL report an authentication error for that plugin and continue processing other plugins
