## MODIFIED Requirements

### Requirement: Global y table is exposed to Lua

The system SHALL expose a global table named `y` to the Lua environment. This table serves as the namespace for all yeet configuration APIs. The `y` table SHALL contain a `theme` subtable for static theme configuration and a `hook` subtable for callback functions.

#### Scenario: y table is accessible in init.lua

- **WHEN** `init.lua` contains `y.theme.StatusLineFg = '#ffffff'`
- **THEN** the assignment executes without error and the value is accessible from the Rust side

#### Scenario: y.hook subtable is accessible in init.lua

- **WHEN** `init.lua` contains `y.hook.on_window_create = function(ctx) end`
- **THEN** the assignment executes without error and the function is retained in the Lua runtime for later invocation
