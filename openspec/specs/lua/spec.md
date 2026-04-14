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

### Requirement: on_window_change hook fires at the end of each function that changes viewport paths or buffers

The system SHALL invoke `y.hook.on_window_change` at the end of each public function that actually changes viewport paths or buffer assignments. The affected functions are: `navigate::mark`, `navigate::path`, `navigate::path_as_preview`, `navigate::navigate_to_path_with_selection`, `navigate::parent`, `navigate::selected`, `cursor::relocate` (Directory branch), `viewport::relocate` (Directory branch), `enumeration::change`, `enumeration::finish`, `path::add`, `path::remove`, `modify::buffer` (Directory branch). The hook SHALL fire for Directory windows only.

#### Scenario: Preview changes from directory to file

- **WHEN** the user navigates in a Directory window and the preview target changes from a directory to a file
- **THEN** the system SHALL invoke `y.hook.on_window_change` with a context table where `ctx.type == "directory"` and `ctx.preview_is_directory == false`

#### Scenario: Preview changes from file to directory

- **WHEN** the user navigates in a Directory window and the preview target changes from a file to a directory
- **THEN** the system SHALL invoke `y.hook.on_window_change` with a context table where `ctx.type == "directory"` and `ctx.preview_is_directory == true`

#### Scenario: Navigation changes all viewports

- **WHEN** the user navigates to a parent or child directory (e.g., `navigate::parent`, `navigate::selected`)
- **THEN** the system SHALL invoke `y.hook.on_window_change` at the end of that function after all viewport swaps and buffer assignments are complete

#### Scenario: Cursor movement triggers hook

- **WHEN** the user moves the cursor to a different entry in the current directory
- **THEN** the system SHALL invoke `y.hook.on_window_change` at the end of the cursor/viewport relocate function after the preview updates

#### Scenario: Preview changes to empty

- **WHEN** the user navigates to an empty directory where there is no preview target
- **THEN** the system SHALL invoke `y.hook.on_window_change` with a context table where `ctx.preview_is_directory == false`

### Requirement: on_window_change context table structure

The context table for `on_window_change` SHALL contain the same fields as `on_window_create` for Directory windows: `type`, `parent`, `current`, and `preview` viewport settings tables. Additionally, the context table SHALL contain a `preview_is_directory` boolean field indicating whether the current preview target is a directory buffer. Each viewport subtable (`parent`, `current`, `preview`) SHALL include a `path` string field set to the resolved path of that viewport's buffer. The top-level `path` field SHALL NOT be present on the `on_window_change` context table.

#### Scenario: Context contains per-viewport paths

- **WHEN** `y.hook.on_window_change` is invoked for a Directory window
- **THEN** the context table SHALL have the structure `{ type = "directory", parent = { path = "<parent_dir>", <viewport_settings> }, current = { path = "<current_dir>", <viewport_settings> }, preview = { path = "<preview_target>", <viewport_settings> }, preview_is_directory = <boolean> }`

#### Scenario: Parent path is the parent directory

- **WHEN** the user is navigating in `/home/user/projects/myapp`
- **THEN** `ctx.parent.path` SHALL be the parent directory path (e.g., `/home/user/projects`)

#### Scenario: Current path is the current directory

- **WHEN** the user is navigating in `/home/user/projects/myapp`
- **THEN** `ctx.current.path` SHALL be `/home/user/projects/myapp`

#### Scenario: Preview path is the preview target

- **WHEN** the preview is showing `/home/user/projects/myapp/src`
- **THEN** `ctx.preview.path` SHALL be `/home/user/projects/myapp/src`

#### Scenario: Preview path for file preview

- **WHEN** the preview is showing file `/home/user/projects/myapp/README.md`
- **THEN** `ctx.preview.path` SHALL be `/home/user/projects/myapp/README.md`

#### Scenario: Path is nil when buffer has no path

- **WHEN** a viewport's buffer cannot resolve a path
- **THEN** the corresponding viewport subtable's `path` field SHALL be nil

#### Scenario: Top-level path is absent

- **WHEN** `y.hook.on_window_change` is invoked
- **THEN** `ctx.path` SHALL be nil (not present on the context table)

#### Scenario: preview_is_directory is true for directory preview

- **WHEN** the preview target is a directory buffer
- **THEN** `ctx.preview_is_directory` SHALL be `true`

#### Scenario: preview_is_directory is false for file preview

- **WHEN** the preview target is a content buffer (file)
- **THEN** `ctx.preview_is_directory` SHALL be `false`

#### Scenario: Path properties are read-only

- **WHEN** a callback modifies `ctx.current.path` to a different value
- **THEN** the system SHALL NOT read back the path change; viewport paths are informational only

### Requirement: on_window_change hook mutations are applied to viewports

Modifications to viewport settings fields in the context table SHALL be read back and applied to the corresponding ViewPort structs after all callbacks return. The read-back semantics SHALL be identical to `on_window_create`.

#### Scenario: Hook modifies prefix_column_width on preview

- **WHEN** the hook sets `ctx.preview.prefix_column_width = 2`
- **THEN** the preview viewport's `prefix_column_width` field SHALL be set to `2`

#### Scenario: Hook modifies wrap on preview

- **WHEN** the hook sets `ctx.preview.wrap = true`
- **THEN** the preview viewport's `wrap` field SHALL be set to `true`

### Requirement: on_window_change cycle prevention

The `on_window_change` hook SHALL NOT re-fire as a result of viewport setting changes made by hook callbacks. Cycle prevention is achieved by invocation placement: the hook fires at the end of each function, after all mutations within that function are complete. Viewport modifications by callbacks do not trigger additional function calls or re-invoke the hook.

#### Scenario: Hook modifies viewport settings without re-triggering

- **WHEN** an `on_window_change` callback sets `ctx.preview.prefix_column_width = 2`
- **THEN** the system SHALL apply the change and NOT invoke `on_window_change` again

### Requirement: on_window_change hook only fires for Directory windows

The `on_window_change` hook SHALL only fire for Directory windows. Help, QuickFix, and Tasks windows SHALL NOT trigger this hook.

#### Scenario: Help window does not trigger on_window_change

- **WHEN** a Help window is active
- **THEN** the system SHALL NOT invoke `y.hook.on_window_change`

### Requirement: on_window_change helper function centralizes invocation logic

The system SHALL provide an `invoke_on_window_change_for_focused` helper function in `yeet-frontend/src/update/hook.rs` that encapsulates the repeated invocation logic: get focused directory buffer IDs, resolve current path, determine `preview_is_directory`, get mutable viewports, and call `yeet_lua::invoke_on_window_change`. Each affected function SHALL call this helper to avoid code duplication.

#### Scenario: Helper resolves buffer IDs and invokes hook

- **WHEN** a public function calls `invoke_on_window_change_for_focused` with the model and lua configuration
- **THEN** the helper SHALL resolve the focused directory buffer IDs, determine whether the preview is a directory, and invoke the Lua hook with the correct context

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
