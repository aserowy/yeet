## ADDED Requirements

### Requirement: on_window_change hook fires when preview target changes

The system SHALL invoke `y.hook.on_window_change` whenever the preview buffer assignment changes in a Directory window. The hook SHALL fire after the preview buffer ID is updated and after `hide_cursor_line` is set based on whether the preview target is a directory.

#### Scenario: Preview changes from directory to file

- **WHEN** the user navigates in a Directory window and the preview target changes from a directory to a file
- **THEN** the system SHALL invoke `y.hook.on_window_change` with a context table where `ctx.type == "directory"` and `ctx.preview_is_directory == false`

#### Scenario: Preview changes from file to directory

- **WHEN** the user navigates in a Directory window and the preview target changes from a file to a directory
- **THEN** the system SHALL invoke `y.hook.on_window_change` with a context table where `ctx.type == "directory"` and `ctx.preview_is_directory == true`

#### Scenario: Preview changes between two directories

- **WHEN** the user navigates in a Directory window and the preview target changes from one directory to another directory
- **THEN** the system SHALL invoke `y.hook.on_window_change` with a context table where `ctx.preview_is_directory == true`

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

The `on_window_change` hook SHALL NOT re-fire as a result of viewport setting changes made by hook callbacks. The hook SHALL only fire in response to external state changes (preview buffer assignment changes).

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
