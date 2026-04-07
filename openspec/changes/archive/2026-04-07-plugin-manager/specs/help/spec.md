## ADDED Requirements

### Requirement: Plugins help page

The system SHALL include a `plugins` help page accessible via `:help plugins`. The page SHALL document plugin registration (`y.plugin.register()`), plugin commands (`:pluginlist`, `:pluginsync`, `:pluginupdate`), the lock file, concurrency configuration, plugin storage, startup behavior, cleanup, and plugin authoring.

#### Scenario: Help plugins page exists

- **WHEN** the user executes `:help plugins`
- **THEN** the system SHALL open the help window displaying the plugins documentation

#### Scenario: Help topic resolves plugin entries

- **WHEN** the user executes `:help pluginlist`
- **THEN** the system SHALL open the plugins help page scrolled to the `pluginlist` entry

#### Scenario: Help topic resolves pluginsync

- **WHEN** the user executes `:help pluginsync`
- **THEN** the system SHALL open the plugins help page scrolled to the `pluginsync` entry

#### Scenario: Help topic resolves pluginupdate

- **WHEN** the user executes `:help pluginupdate`
- **THEN** the system SHALL open the plugins help page scrolled to the `pluginupdate` entry

### Requirement: Help index lists plugins page

The help index page SHALL include an entry for the `plugins` page with a brief description.

#### Scenario: Index shows plugins entry

- **WHEN** the user executes `:help`
- **THEN** the index page SHALL contain an entry for `plugins` with a description of plugin management
