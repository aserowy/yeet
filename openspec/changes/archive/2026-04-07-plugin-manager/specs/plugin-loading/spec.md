## ADDED Requirements

### Requirement: Plugins are loaded on startup after init.lua

The system SHALL load plugins after `init.lua` has been executed and the plugin registration list has been built. Loading SHALL occur before UI rendering begins.

#### Scenario: Plugins loaded before first render

- **WHEN** yeet starts with registered plugins that are all present on disk
- **THEN** all plugin `init.lua` files SHALL be executed before the first UI render

#### Scenario: No plugins registered

- **WHEN** yeet starts and no plugins have been registered via `y.plugin.register()`
- **THEN** the system SHALL skip the plugin loading phase and proceed to UI rendering

### Requirement: Plugin loading executes init.lua

For each registered plugin, the system SHALL look for an `init.lua` file in the plugin's data directory and execute it in the Lua runtime. Plugins SHALL be loaded in registration order.

#### Scenario: Plugin with init.lua

- **WHEN** plugin A is registered and its directory contains `init.lua`
- **THEN** the system SHALL execute `init.lua` in the Lua runtime

#### Scenario: Plugin directory exists but has no init.lua

- **WHEN** plugin A is registered and its directory exists but contains no `init.lua`
- **THEN** the system SHALL record the plugin as `error` with a message indicating missing `init.lua`

### Requirement: Dependencies are loaded before dependents

The system SHALL load plugin dependencies before the plugin that declares them. If dependency B is required by plugin A, B's `init.lua` SHALL execute before A's `init.lua`.

#### Scenario: Plugin with one dependency

- **WHEN** plugin A depends on library B and both are present on disk
- **THEN** library B's `init.lua` SHALL execute before plugin A's `init.lua`

#### Scenario: Shared dependency loaded once

- **WHEN** plugin A and plugin B both depend on library C
- **THEN** library C's `init.lua` SHALL execute exactly once, before whichever dependent is loaded first

### Requirement: Only downloaded plugins are loaded

The system SHALL only attempt to load plugins whose directories exist in the data directory. Missing plugins SHALL NOT trigger any download or network operation.

#### Scenario: Plugin directory missing

- **WHEN** plugin A is registered but its directory does not exist in the data folder
- **THEN** the system SHALL record plugin A as `missing` and skip loading it

#### Scenario: Some plugins present, some missing

- **WHEN** plugins A and B are registered, A exists on disk but B does not
- **THEN** the system SHALL load A and record B as `missing`

### Requirement: Missing plugins reported as error on startup

After the plugin loading phase, if any registered plugins are missing from disk, the system SHALL emit a single error message listing all missing plugins.

#### Scenario: Two missing plugins

- **WHEN** plugins A and B are both registered but neither directory exists
- **THEN** the system SHALL emit one error message listing both A and B as missing

#### Scenario: All plugins present

- **WHEN** all registered plugins exist on disk
- **THEN** the system SHALL NOT emit any missing plugin error

### Requirement: Failed plugin init.lua is rolled back

If a plugin's `init.lua` raises an error during execution, the system SHALL roll back the Lua state to a snapshot taken before that plugin's execution. No partial side effects (hooks, theme changes) from the failed plugin SHALL persist.

#### Scenario: Plugin with syntax error

- **WHEN** plugin A's `init.lua` contains a syntax error
- **THEN** the Lua state SHALL be rolled back to before plugin A's execution, and plugin A SHALL be recorded as `error` with the error message

#### Scenario: Plugin with runtime error after registering a hook

- **WHEN** plugin A's `init.lua` registers a hook via `y.hook.on_window_create:add(fn)` and then raises a runtime error
- **THEN** the hook registration SHALL be rolled back and the hook list SHALL not contain the function from plugin A

#### Scenario: Failed plugin does not block others

- **WHEN** plugin A fails during loading and plugin B is next in registration order
- **THEN** plugin B's `init.lua` SHALL still be executed

### Requirement: Failed dependency marks dependent as error

If a dependency fails to load (error or missing), any plugin that depends on it SHALL also be marked as `error` with a message indicating the failed dependency. The dependent plugin's `init.lua` SHALL NOT be executed.

#### Scenario: Dependency fails, dependent skipped

- **WHEN** dependency B fails to load and plugin A depends on B
- **THEN** plugin A SHALL be recorded as `error` with a message referencing B's failure, and A's `init.lua` SHALL NOT execute

#### Scenario: Dependency missing, dependent skipped

- **WHEN** dependency B is missing from disk and plugin A depends on B
- **THEN** plugin A SHALL be recorded as `error` with a message referencing B being missing

### Requirement: Per-plugin state tracking

The system SHALL maintain a `PluginState` for each registered plugin in memory for the application lifetime. Each state SHALL contain:

- Plugin name/URL
- Status: one of `loaded`, `error`, `missing`
- Error message (if status is `error` or `missing`)
- Resolved version/commit (if known)

#### Scenario: Plugin loaded successfully

- **WHEN** plugin A loads without error
- **THEN** its state SHALL be `loaded` with no error message

#### Scenario: Plugin failed to load

- **WHEN** plugin A's `init.lua` raises an error
- **THEN** its state SHALL be `error` with the error message from the Lua runtime

#### Scenario: Plugin missing from disk

- **WHEN** plugin A's directory does not exist
- **THEN** its state SHALL be `missing` with a message indicating the expected path
