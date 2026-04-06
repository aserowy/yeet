## MODIFIED Requirements

### Requirement: Hook namespace on y table

The system SHALL expose a `y.hook` table in the Lua environment. Each hook name (`on_window_create`) SHALL be a Lua table with an `:add()` method for registering callback functions. The table SHALL store registered callbacks in order.

#### Scenario: y.hook table exists before init.lua executes

- **WHEN** the Lua runtime initializes and before `init.lua` is executed
- **THEN** `y.hook` SHALL be a table with `on_window_create` as a hook object supporting `:add()`

#### Scenario: User registers a callback via :add()

- **WHEN** `init.lua` contains `y.hook.on_window_create:add(function(ctx) end)`
- **THEN** the callback is appended to the hook's internal list and retained in the Lua runtime

#### Scenario: User registers multiple callbacks

- **WHEN** `init.lua` calls `y.hook.on_window_create:add(fn1)` and then `y.hook.on_window_create:add(fn2)`
- **THEN** both callbacks SHALL be stored in registration order

### Requirement: Hook invocation checks for function presence

The system SHALL iterate all registered callbacks for a hook when invoking it. If no callbacks are registered, the system SHALL skip invocation silently. Each registered entry SHALL be validated as a function before calling.

#### Scenario: No callbacks registered

- **WHEN** `y.hook.on_window_create` has no registered callbacks and a window is created
- **THEN** the system SHALL skip invocation and use default viewport settings

#### Scenario: Multiple callbacks registered

- **WHEN** `y.hook.on_window_create` has two registered callbacks and a window is created
- **THEN** the system SHALL invoke both callbacks in registration order

#### Scenario: Callbacks share context table

- **WHEN** multiple callbacks are registered and the first callback sets `ctx.current.wrap = true`
- **THEN** the second callback SHALL see `ctx.current.wrap == true` in the same context table

### Requirement: Hook errors are handled gracefully

The system SHALL NOT crash if a hook callback raises a Lua error. The system SHALL log the error and continue invoking the remaining callbacks.

#### Scenario: First callback errors, second callback still runs

- **WHEN** two callbacks are registered and the first raises a runtime error
- **THEN** the system SHALL log the error from the first callback and still invoke the second callback

#### Scenario: All callbacks error

- **WHEN** all registered callbacks raise errors
- **THEN** the system SHALL log each error and apply default viewport settings

### Requirement: Hook context table read-back with validation

The system SHALL read back viewport settings from the context table after all registered callbacks have been invoked. Unknown keys SHALL be ignored. Invalid values SHALL be ignored with a warning logged, preserving the default value for that field.

#### Scenario: Hook sets a valid viewport field

- **WHEN** a callback sets `ctx.current.wrap = true` on a directory window context
- **THEN** the system SHALL apply `wrap = true` to the current viewport

#### Scenario: Hook sets an invalid type for a viewport field

- **WHEN** a callback sets `ctx.current.line_number = 42` (integer instead of string)
- **THEN** the system SHALL log a warning and keep the default `line_number` value for that viewport

#### Scenario: Hook sets an unrecognized enum value

- **WHEN** a callback sets `ctx.current.line_number = "fancy"`
- **THEN** the system SHALL log a warning and keep the default `line_number` value for that viewport

#### Scenario: Hook adds an unknown key to the context

- **WHEN** a callback sets `ctx.current.unknown_field = true`
- **THEN** the system SHALL ignore the unknown key without error or warning

### Requirement: :add() rejects non-function arguments

The `:add()` method SHALL only accept function arguments. If called with a non-function value, the system SHALL log a warning and not add the value to the callback list.

#### Scenario: :add() called with a string

- **WHEN** `y.hook.on_window_create:add("not a function")` is called
- **THEN** the system SHALL log a warning and the callback list SHALL remain unchanged

#### Scenario: :add() called with nil

- **WHEN** `y.hook.on_window_create:add(nil)` is called
- **THEN** the system SHALL log a warning and the callback list SHALL remain unchanged
