### Requirement: Lua runtime initialization
The system SHALL embed a Lua 5.4 runtime using the mlua crate at application startup, before any UI rendering occurs.

#### Scenario: Successful initialization without config file
- **WHEN** yeet starts and no `init.lua` exists at the config path
- **THEN** the Lua runtime is initialized but no user script is executed, and yeet starts with default settings

#### Scenario: Successful initialization with config file
- **WHEN** yeet starts and `init.lua` exists at `$XDG_CONFIG_HOME/yeet/init.lua`
- **THEN** the Lua runtime loads and executes the file before UI rendering begins

### Requirement: Config file location follows XDG
The system SHALL look for `init.lua` at `$XDG_CONFIG_HOME/yeet/init.lua`. If `$XDG_CONFIG_HOME` is not set, the system SHALL fall back to `~/.config/yeet/init.lua`.

#### Scenario: XDG_CONFIG_HOME is set
- **WHEN** `$XDG_CONFIG_HOME` is set to `/custom/config`
- **THEN** the system loads `/custom/config/yeet/init.lua`

#### Scenario: XDG_CONFIG_HOME is not set
- **WHEN** `$XDG_CONFIG_HOME` is not set and the user's home directory is `/home/user`
- **THEN** the system loads `/home/user/.config/yeet/init.lua`

### Requirement: Global y table is exposed to Lua
The system SHALL expose a global table named `y` to the Lua environment. This table serves as the namespace for all yeet configuration APIs. The `y` table SHALL contain a `theme` subtable for static theme configuration, a `hook` subtable for callback functions, and a `plugin` subtable for plugin management. The `y` table SHALL be protected from overwrite: assigning a table to `y` at the global level SHALL shallow-merge the new table's keys into the existing `y` table instead of replacing it.

#### Scenario: y table is accessible in init.lua
- **WHEN** `init.lua` contains `y.theme.StatusLineFg = '#ffffff'`
- **THEN** the assignment executes without error and the value is accessible from the Rust side

#### Scenario: y.hook subtable is accessible in init.lua
- **WHEN** `init.lua` contains `y.hook.on_window_create:add(function(ctx) end)`
- **THEN** the assignment executes without error and the function is retained in the Lua runtime for later invocation

#### Scenario: y.plugin subtable is accessible in init.lua
- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/plugin" })`
- **THEN** the call executes without error and the plugin is added to the registration list

#### Scenario: y.plugin.concurrency is settable
- **WHEN** `init.lua` contains `y.plugin.concurrency = 2`
- **THEN** the value is accessible from the Rust side as an integer

#### Scenario: Wholesale y assignment merges instead of replacing
- **WHEN** `init.lua` contains `y = { theme = { TabBarActiveBg = "#ff0000" } }`
- **THEN** `y.theme.TabBarActiveBg` SHALL be `"#ff0000"` and `y.hook` and `y.plugin` SHALL still exist with their methods intact

#### Scenario: y.hook survives y table reassignment
- **WHEN** `init.lua` contains `y = { theme = { ... } }` followed by `y.hook.on_window_create:add(function(ctx) end)`
- **THEN** the hook registration SHALL succeed without error

#### Scenario: y.plugin survives y table reassignment
- **WHEN** `init.lua` contains `y = { theme = { ... } }` followed by `y.plugin.register({ url = "..." })`
- **THEN** the plugin registration SHALL succeed without error

#### Scenario: Non-table assignment to y is ignored
- **WHEN** `init.lua` contains `y = nil` or `y = "string"`
- **THEN** the system SHALL log a warning and the existing `y` table SHALL remain unchanged

### Requirement: Lua errors are reported gracefully
The system SHALL NOT crash if `init.lua` contains syntax errors or runtime errors. The system SHALL log the error and continue startup with default settings.

#### Scenario: Syntax error in init.lua
- **WHEN** `init.lua` contains invalid Lua syntax
- **THEN** yeet logs an error message indicating the file and error, and starts with default theme colors

#### Scenario: Runtime error in init.lua
- **WHEN** `init.lua` raises a runtime error (e.g., calling a nil value)
- **THEN** yeet logs the error with a stack trace and starts with default theme colors

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

### Requirement: :add() rejects non-function arguments

The `:add()` method SHALL only accept function arguments. If called with a non-function value, the system SHALL log a warning and not add the value to the callback list.

#### Scenario: :add() called with a string

- **WHEN** `y.hook.on_window_create:add("not a function")` is called
- **THEN** the system SHALL log a warning and the callback list SHALL remain unchanged

#### Scenario: :add() called with nil

- **WHEN** `y.hook.on_window_create:add(nil)` is called
- **THEN** the system SHALL log a warning and the callback list SHALL remain unchanged

### Requirement: on_window_create hook fires for Directory windows

The system SHALL invoke `y.hook.on_window_create` whenever a Directory window is created. The context table SHALL contain the window type, target path, and viewport settings for all three panes (parent, current, preview).

#### Scenario: Directory window created on startup

- **WHEN** yeet starts and creates the initial Directory window
- **THEN** the system SHALL invoke `y.hook.on_window_create` with a context table where `ctx.type == "directory"` and `ctx.parent`, `ctx.current`, and `ctx.preview` contain viewport settings

#### Scenario: Directory window created via new tab

- **WHEN** a new tab is created
- **THEN** the system SHALL invoke `y.hook.on_window_create` with `ctx.type == "directory"`

#### Scenario: Directory window created via split

- **WHEN** a horizontal or vertical split is created
- **THEN** the system SHALL invoke `y.hook.on_window_create` with `ctx.type == "directory"` for the newly created Directory window

#### Scenario: Directory window created from quickfix open

- **WHEN** a quickfix entry is opened and a new Directory window is created via split
- **THEN** the system SHALL invoke `y.hook.on_window_create` with `ctx.type == "directory"`

### Requirement: on_window_create hook fires for Help windows

The system SHALL invoke `y.hook.on_window_create` whenever a Help window is created.

#### Scenario: Help window created

- **WHEN** the help command is executed and a Help window is created
- **THEN** the system SHALL invoke `y.hook.on_window_create` with a context table where `ctx.type == "help"` and `ctx.viewport` contains the viewport settings

### Requirement: on_window_create hook fires for QuickFix windows

The system SHALL invoke `y.hook.on_window_create` whenever a QuickFix window is created.

#### Scenario: QuickFix window created

- **WHEN** the copen command is executed and a QuickFix window is created
- **THEN** the system SHALL invoke `y.hook.on_window_create` with a context table where `ctx.type == "quickfix"` and `ctx.viewport` contains the viewport settings

### Requirement: on_window_create hook fires for Tasks windows

The system SHALL invoke `y.hook.on_window_create` whenever a Tasks window is created.

#### Scenario: Tasks window created

- **WHEN** the topen command is executed and a Tasks window is created
- **THEN** the system SHALL invoke `y.hook.on_window_create` with a context table where `ctx.type == "tasks"` and `ctx.viewport` contains the viewport settings

### Requirement: Directory context table structure

For Directory windows, the context table SHALL contain `type`, `path`, `parent`, `current`, and `preview` fields. The `parent`, `current`, and `preview` fields SHALL each be a viewport settings table.

#### Scenario: Directory context contains all pane settings

- **WHEN** `y.hook.on_window_create` is invoked for a Directory window
- **THEN** the context table SHALL have the structure `{ type = "directory", path = "<target_path>", parent = { <viewport_settings> }, current = { <viewport_settings> }, preview = { <viewport_settings> } }`

#### Scenario: Directory context path reflects target

- **WHEN** a Directory window is created targeting `/home/user/projects`
- **THEN** `ctx.path` SHALL be `"/home/user/projects"` if the path is known at creation time, or nil if not yet determined

### Requirement: Single-viewport context table structure

For Help, QuickFix, and Tasks windows, the context table SHALL contain `type` and `viewport` fields. The `viewport` field SHALL be a viewport settings table.

#### Scenario: Help context structure

- **WHEN** `y.hook.on_window_create` is invoked for a Help window
- **THEN** the context table SHALL have the structure `{ type = "help", viewport = { <viewport_settings> } }`

#### Scenario: QuickFix context structure

- **WHEN** `y.hook.on_window_create` is invoked for a QuickFix window
- **THEN** the context table SHALL have the structure `{ type = "quickfix", viewport = { <viewport_settings> } }`

#### Scenario: Tasks context structure

- **WHEN** `y.hook.on_window_create` is invoked for a Tasks window
- **THEN** the context table SHALL have the structure `{ type = "tasks", viewport = { <viewport_settings> } }`

### Requirement: Viewport settings table fields

Each viewport settings table SHALL contain the following fields reflecting the current viewport defaults: `line_number`, `line_number_width`, `sign_column_width`, `show_border`, `hide_cursor`, `hide_cursor_line`, and `wrap`.

#### Scenario: Viewport settings table contains all writable fields

- **WHEN** a viewport settings table is passed in the context
- **THEN** it SHALL contain `line_number` (string: "none", "absolute", or "relative"), `line_number_width` (integer), `sign_column_width` (integer), `show_border` (boolean), `hide_cursor` (boolean), `hide_cursor_line` (boolean), and `wrap` (boolean)

#### Scenario: Viewport settings reflect pre-hook defaults

- **WHEN** the hook is invoked for a new Directory window
- **THEN** `ctx.current.line_number` SHALL be `"relative"`, `ctx.current.line_number_width` SHALL be `3`, `ctx.current.show_border` SHALL be `true`, and `ctx.current.sign_column_width` SHALL be `2` (matching the hardcoded defaults in `Window::create`)

### Requirement: Hook mutations are applied to viewports

Modifications to viewport settings fields in the context table SHALL be read back and applied to the corresponding ViewPort structs after the hook returns.

#### Scenario: Hook modifies line_number on current pane

- **WHEN** the hook sets `ctx.current.line_number = "absolute"` for a Directory window
- **THEN** the current viewport's `line_number` field SHALL be set to `LineNumber::Absolute`

#### Scenario: Hook modifies wrap on preview pane

- **WHEN** the hook sets `ctx.preview.wrap = true` for a Directory window
- **THEN** the preview viewport's `wrap` field SHALL be set to `true`

#### Scenario: Hook modifies hide_cursor on single-viewport window

- **WHEN** the hook sets `ctx.viewport.hide_cursor = false` for a Help window
- **THEN** the help viewport's `hide_cursor` field SHALL be set to `false`

#### Scenario: Hook modifies sign_column_width

- **WHEN** the hook sets `ctx.current.sign_column_width = 4` for a Directory window
- **THEN** the current viewport's `sign_column_width` field SHALL be set to `4`

#### Scenario: Hook modifies show_border

- **WHEN** the hook sets `ctx.parent.show_border = false` for a Directory window
- **THEN** the parent viewport's `show_border` field SHALL be set to `false`

### Requirement: Hook invocation occurs before layout

The system SHALL invoke `y.hook.on_window_create` after the window is constructed with default viewport settings and before layout dimensions are assigned.

#### Scenario: Settings applied before first render

- **WHEN** a window is created and the hook modifies viewport settings
- **THEN** the modified settings SHALL be in effect before the window receives its first layout calculation and render
