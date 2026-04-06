## MODIFIED Requirements

### Requirement: Global y table is exposed to Lua

The system SHALL expose a global table named `y` to the Lua environment. This table serves as the namespace for all yeet configuration APIs. The `y` table SHALL contain a `theme` subtable for static theme configuration and a `hook` subtable for callback functions. The `y` table SHALL be protected from overwrite: assigning a table to `y` at the global level SHALL shallow-merge the new table's keys into the existing `y` table instead of replacing it.

#### Scenario: y table is accessible in init.lua

- **WHEN** `init.lua` contains `y.theme.StatusLineFg = '#ffffff'`
- **THEN** the assignment executes without error and the value is accessible from the Rust side

#### Scenario: y.hook subtable is accessible in init.lua

- **WHEN** `init.lua` contains `y.hook.on_window_create:add(function(ctx) end)`
- **THEN** the assignment executes without error and the function is retained in the Lua runtime for later invocation

#### Scenario: Wholesale y assignment merges instead of replacing

- **WHEN** `init.lua` contains `y = { theme = { TabBarActiveBg = "#ff0000" } }`
- **THEN** `y.theme.TabBarActiveBg` SHALL be `"#ff0000"` and `y.hook` SHALL still exist with its `:add()` method intact

#### Scenario: y.hook survives y table reassignment

- **WHEN** `init.lua` contains `y = { theme = { ... } }` followed by `y.hook.on_window_create:add(function(ctx) end)`
- **THEN** the hook registration SHALL succeed without error

#### Scenario: Non-table assignment to y is ignored

- **WHEN** `init.lua` contains `y = nil` or `y = "string"`
- **THEN** the system SHALL log a warning and the existing `y` table SHALL remain unchanged
