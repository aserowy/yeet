## MODIFIED Requirements

### Requirement: Hook namespace on y table

The system SHALL expose a `y.hook` table in the Lua environment. Each hook name (`on_window_create`, `on_window_change`, `on_bufferline_mutate`) SHALL be a Lua table with an `:add()` method for registering callback functions. The table SHALL store registered callbacks in order.

#### Scenario: y.hook table exists before init.lua executes

- **WHEN** the Lua runtime initializes and before `init.lua` is executed
- **THEN** `y.hook` SHALL be a table with `on_window_create`, `on_window_change`, and `on_bufferline_mutate` as hook objects supporting `:add()`

#### Scenario: User registers a callback via :add()

- **WHEN** `init.lua` contains `y.hook.on_window_create:add(function(ctx) end)`
- **THEN** the callback is appended to the hook's internal list and retained in the Lua runtime

#### Scenario: User registers multiple callbacks

- **WHEN** `init.lua` calls `y.hook.on_window_create:add(fn1)` and then `y.hook.on_window_create:add(fn2)`
- **THEN** both callbacks SHALL be stored in registration order

#### Scenario: User registers on_window_change callback

- **WHEN** `init.lua` contains `y.hook.on_window_change:add(function(ctx) end)`
- **THEN** the callback is appended to the on_window_change hook's internal list and retained in the Lua runtime
