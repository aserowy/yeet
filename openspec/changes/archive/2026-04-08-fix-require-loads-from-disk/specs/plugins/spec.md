## MODIFIED Requirements

### Requirement: Plugin loading supports require()

#### Scenario: require() loads plugin from disk when available

- **WHEN** a plugin is registered and its directory exists on disk with `init.lua`, and `require('plugin-name')` is called during `init.lua`
- **THEN** the searcher SHALL load the plugin's `init.lua` via `dofile()`, store the result in `package.loaded`, and return the real module table

#### Scenario: require() returns proxy when plugin not on disk

- **WHEN** a plugin is registered but its directory does not exist on disk (fresh install), and `require('plugin-name')` is called during `init.lua`
- **THEN** the searcher SHALL return a no-op proxy table

#### Scenario: Plugin loaded via require() is not double-loaded

- **WHEN** a plugin was already loaded via `require()` during `init.lua` and `load_plugins` runs afterward
- **THEN** `load_plugins` SHALL skip re-executing that plugin's `init.lua`
