## ADDED Requirements

### Requirement: require() returns no-op proxy for unloaded plugins

When `require()` is called for a module name matching a registered plugin that is not yet loaded, the system SHALL return a no-op proxy table instead of raising an error. Any method call on the proxy SHALL silently do nothing. Once the plugin is loaded (on subsequent startup after sync/update), `require()` SHALL return the real module table.

#### Scenario: require() on fresh install

- **WHEN** `init.lua` calls `require('bluloco-theme').setup()` and the plugin is registered but not yet downloaded
- **THEN** the `require()` call SHALL return a no-op proxy and `setup()` SHALL silently do nothing

#### Scenario: require() after plugin is loaded

- **WHEN** the plugin has been synced/updated, the app restarts, and `require('bluloco-theme')` is called
- **THEN** `require()` SHALL return the real module table from `package.loaded`

#### Scenario: require() for unknown module still errors

- **WHEN** `require('nonexistent-module')` is called and no plugin with that name is registered
- **THEN** `require()` SHALL raise the standard Lua error
