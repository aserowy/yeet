## ADDED Requirements

### Requirement: Plugin registration via y.plugin.register()

The system SHALL expose a `y.plugin.register(opts)` function in the Lua environment. The `opts` table SHALL accept the following fields:

- `url` (string, required): Git repository URL
- `branch` (string, optional): Branch name, defaults to remote HEAD
- `version` (string, optional): Semver tag range constraint (e.g., `">=1.0, <2.0"`)
- `dependencies` (table, optional): Array of dependency tables using the same opts shape (without nested dependencies)

Each call to `register()` SHALL append the plugin specification to an in-memory list. No network calls SHALL occur during registration.

#### Scenario: Register a plugin with all options

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-nord", branch = "main", version = ">=1.0, <2.0" })`
- **THEN** the plugin specification SHALL be stored in memory with the given URL, branch, and version constraint

#### Scenario: Register a plugin with only URL

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-nord" })`
- **THEN** the plugin specification SHALL be stored with branch defaulting to remote HEAD and no version constraint

#### Scenario: Register a plugin with dependencies

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-theme", dependencies = { { url = "https://github.com/user/yeet-lib" } } })`
- **THEN** the plugin specification SHALL be stored with one dependency entry

#### Scenario: Register called without URL

- **WHEN** `init.lua` contains `y.plugin.register({ branch = "main" })`
- **THEN** the system SHALL log an error and not add the entry to the plugin list

#### Scenario: Register called with non-table argument

- **WHEN** `init.lua` contains `y.plugin.register("https://github.com/user/plugin")`
- **THEN** the system SHALL log an error and not add the entry to the plugin list

### Requirement: Plugin list is readable from Rust

The system SHALL provide a function to read the registered plugin list from the Lua runtime as a Vec of plugin specification structs. This list SHALL include all plugins and their dependencies.

#### Scenario: Reading plugins after registration

- **WHEN** two plugins have been registered via `y.plugin.register()` and Rust reads the plugin list
- **THEN** the returned list SHALL contain two entries in registration order with their respective fields

#### Scenario: Reading plugins when none registered

- **WHEN** no plugins have been registered and Rust reads the plugin list
- **THEN** the returned list SHALL be empty

### Requirement: Dependency deduplication by URL

When multiple plugins declare the same dependency URL, the system SHALL treat them as a single dependency. If version constraints differ, the system SHALL use the most restrictive intersection.

#### Scenario: Same dependency declared by two plugins

- **WHEN** plugin A declares dependency `{ url = "https://github.com/user/lib", version = ">=1.0" }` and plugin B declares dependency `{ url = "https://github.com/user/lib", version = "<2.0" }`
- **THEN** the system SHALL resolve a single dependency with constraint `">=1.0, <2.0"`

#### Scenario: Duplicate dependency with identical opts

- **WHEN** plugin A and plugin B both declare dependency `{ url = "https://github.com/user/lib" }`
- **THEN** the system SHALL resolve a single dependency entry

### Requirement: Dependencies cannot have sub-dependencies

The system SHALL ignore any `dependencies` field within a dependency entry. Only top-level plugins may declare dependencies.

#### Scenario: Dependency with nested dependencies

- **WHEN** a plugin registers with `dependencies = { { url = "...", dependencies = { { url = "..." } } } }`
- **THEN** the system SHALL log a warning and ignore the nested `dependencies` field, registering only the direct dependency

### Requirement: Concurrency setting via y.plugin.concurrency

The system SHALL expose `y.plugin.concurrency` as a configurable integer on the `y.plugin` table. The default value SHALL be 4. This value controls the maximum number of parallel git operations during sync and update.

#### Scenario: Setting concurrency

- **WHEN** `init.lua` contains `y.plugin.concurrency = 2`
- **THEN** the system SHALL use 2 as the maximum parallel git operations

#### Scenario: Default concurrency

- **WHEN** `init.lua` does not set `y.plugin.concurrency`
- **THEN** the system SHALL use 4 as the default

#### Scenario: Invalid concurrency value

- **WHEN** `init.lua` contains `y.plugin.concurrency = "fast"`
- **THEN** the system SHALL log a warning and use the default value of 4
