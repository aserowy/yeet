## ADDED Requirements

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

The context table for `on_window_change` SHALL contain the same fields as `on_window_create` for Directory windows: `type`, `path`, `parent`, `current`, and `preview` viewport settings tables. Additionally, the context table SHALL contain a `preview_is_directory` boolean field indicating whether the current preview target is a directory buffer.

#### Scenario: Context contains all viewport settings

- **WHEN** `y.hook.on_window_change` is invoked for a Directory window
- **THEN** the context table SHALL have the structure `{ type = "directory", path = "<current_path>", parent = { <viewport_settings> }, current = { <viewport_settings> }, preview = { <viewport_settings> }, preview_is_directory = <boolean> }`

#### Scenario: preview_is_directory is true for directory preview

- **WHEN** the preview target is a directory buffer
- **THEN** `ctx.preview_is_directory` SHALL be `true`

#### Scenario: preview_is_directory is false for file preview

- **WHEN** the preview target is a content buffer (file)
- **THEN** `ctx.preview_is_directory` SHALL be `false`

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

### Requirement: on_window_change hook namespace registration

The system SHALL register `on_window_change` as a hook object on `y.hook` during Lua runtime initialization, alongside `on_window_create` and `on_bufferline_mutate`. The hook object SHALL support the `:add()` method for registering callbacks.

#### Scenario: on_window_change hook object exists

- **WHEN** the Lua runtime initializes
- **THEN** `y.hook.on_window_change` SHALL be a hook object supporting `:add()`

#### Scenario: Callback registration via :add()

- **WHEN** a plugin calls `y.hook.on_window_change:add(function(ctx) end)`
- **THEN** the callback SHALL be stored in the hook's callback list

### Requirement: on_window_change hook error handling

The system SHALL NOT crash if an `on_window_change` callback raises a Lua error. The system SHALL log the error and continue invoking remaining callbacks, consistent with `on_window_create` error handling.

#### Scenario: Callback error does not crash

- **WHEN** an `on_window_change` callback raises a runtime error
- **THEN** the system SHALL log the error and continue with remaining callbacks

#### Scenario: All callbacks error

- **WHEN** all registered `on_window_change` callbacks raise errors
- **THEN** the system SHALL log each error and apply default viewport settings

### Requirement: Helper function centralizes invocation logic

The system SHALL provide an `invoke_on_window_change_for_focused` helper function in `yeet-frontend/src/update/hook.rs` that encapsulates the repeated invocation logic: get focused directory buffer IDs, resolve current path, determine `preview_is_directory`, get mutable viewports, and call `yeet_lua::invoke_on_window_change`. Each affected function SHALL call this helper to avoid code duplication.

#### Scenario: Helper resolves buffer IDs and invokes hook

- **WHEN** a public function calls `invoke_on_window_change_for_focused` with the model and lua configuration
- **THEN** the helper SHALL resolve the focused directory buffer IDs, determine whether the preview is a directory, and invoke the Lua hook with the correct context
