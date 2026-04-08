## ADDED Requirements

### Requirement: Plugin loading supports require()

The system SHALL add each plugin's directory to Lua's `package.path` before executing the plugin's `init.lua`. If the plugin's `init.lua` returns a non-nil value, the system SHALL store it in `package.loaded` under the plugin's derived name. This enables Lua's standard `require()` to find and return plugin modules.

#### Scenario: Plugin returns a module table

- **WHEN** a plugin's `init.lua` returns a table with a `setup` function
- **THEN** the returned table SHALL be stored in `package.loaded` under the plugin's name and be accessible via `require('plugin-name')`

#### Scenario: Plugin returns nil

- **WHEN** a plugin's `init.lua` does not return a value
- **THEN** `package.loaded` SHALL not be modified for that plugin and the plugin SHALL still be considered loaded

#### Scenario: Plugin name derived from URL

- **WHEN** a plugin is registered with URL `https://github.com/aserowy/yeet-bluloco-theme`
- **THEN** the derived plugin name for `require()` SHALL be `bluloco-theme` (last URL segment with `yeet-` prefix stripped)
