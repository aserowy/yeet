## ADDED Requirements

### Requirement: Hook namespace on y table

The system SHALL expose a `y.hook` table in the Lua environment. This table serves as the namespace for all callback functions that yeet invokes at lifecycle points.

#### Scenario: y.hook table exists before init.lua executes

- **WHEN** the Lua runtime initializes and before `init.lua` is executed
- **THEN** `y.hook` SHALL be an empty table accessible in the Lua environment

#### Scenario: User assigns a function to y.hook

- **WHEN** `init.lua` contains `y.hook.on_window_create = function(ctx) end`
- **THEN** the assignment executes without error and the function is retained in the Lua runtime

### Requirement: Hook invocation checks for function presence

The system SHALL check whether a hook function exists on `y.hook` before invoking it. If the hook is nil or not a function, the system SHALL skip invocation silently without error.

#### Scenario: Hook is nil

- **WHEN** `y.hook.on_window_create` is nil and a window is created
- **THEN** the system SHALL skip the hook invocation and use default viewport settings

#### Scenario: Hook is not a function

- **WHEN** `y.hook.on_window_create` is set to a non-function value (e.g., a string or number)
- **THEN** the system SHALL skip the hook invocation, log a warning, and use default viewport settings

#### Scenario: Hook is a valid function

- **WHEN** `y.hook.on_window_create` is a function
- **THEN** the system SHALL invoke the function with the appropriate context table

### Requirement: Hook errors are handled gracefully

The system SHALL NOT crash if a hook function raises a Lua error. The system SHALL log the error and continue with default values as if the hook was not defined.

#### Scenario: Hook raises a runtime error

- **WHEN** `y.hook.on_window_create` raises a runtime error (e.g., indexing a nil value)
- **THEN** the system SHALL log the error with a stack trace and apply default viewport settings to the created window

#### Scenario: Hook causes an infinite loop or excessive computation

- **WHEN** a hook function does not return in a reasonable time
- **THEN** the system SHALL continue to wait (synchronous execution) — timeout handling is deferred to a future change

### Requirement: Hook context table read-back with validation

The system SHALL read back viewport settings from the context table after a hook function returns. Unknown keys SHALL be ignored. Invalid values (wrong type or unrecognized enum string) SHALL be ignored with a warning logged, preserving the default value for that field.

#### Scenario: Hook sets a valid viewport field

- **WHEN** the hook sets `ctx.current.wrap = true` on a directory window context
- **THEN** the system SHALL apply `wrap = true` to the current viewport

#### Scenario: Hook sets an invalid type for a viewport field

- **WHEN** the hook sets `ctx.current.line_number = 42` (integer instead of string)
- **THEN** the system SHALL log a warning and keep the default `line_number` value for that viewport

#### Scenario: Hook sets an unrecognized enum value

- **WHEN** the hook sets `ctx.current.line_number = "fancy"`
- **THEN** the system SHALL log a warning and keep the default `line_number` value for that viewport

#### Scenario: Hook adds an unknown key to the context

- **WHEN** the hook sets `ctx.current.unknown_field = true`
- **THEN** the system SHALL ignore the unknown key without error or warning

### Requirement: yeet-lua crate encapsulates all Lua logic

All Lua runtime initialization, hook invocation, and Lua table ↔ Rust type conversion logic SHALL reside in the `yeet-lua` crate. The `yeet-lua` crate SHALL be the sole owner of the mlua dependency in the workspace.

#### Scenario: Dependency graph is acyclic

- **WHEN** the workspace is compiled
- **THEN** the dependency graph SHALL be `yeet-buffer` ← `yeet-lua` ← `yeet-frontend` ← `yeet`, with no circular dependencies

#### Scenario: Theme loading moved to yeet-lua

- **WHEN** the application starts
- **THEN** theme loading from `init.lua` SHALL be performed by the `yeet-lua` crate (moved from `yeet/src/lua.rs`)

### Requirement: Lua runtime persists for application lifetime

The system SHALL keep the Lua runtime instance alive for the entire application lifetime. The runtime SHALL be stored in the application model and accessible at all hook invocation points.

#### Scenario: Lua runtime survives past initialization

- **WHEN** `init.lua` has been executed and the UI event loop is running
- **THEN** the Lua runtime instance SHALL still be available for hook invocations

#### Scenario: Lua initialization failure

- **WHEN** the Lua runtime fails to initialize (e.g., out of memory)
- **THEN** the system SHALL store no Lua instance, log the error, and operate with default settings and no hook invocations
