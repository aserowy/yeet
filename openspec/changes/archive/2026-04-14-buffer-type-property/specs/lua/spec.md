## ADDED Requirements

### Requirement: on_window_change viewport subtables include buffer_type property

The system SHALL set a `buffer_type` string property on each viewport subtable (`parent`, `current`, `preview`) in the `on_window_change` context table. The value SHALL be derived from the underlying `Buffer` enum variant for that viewport's buffer. The mapping SHALL be: `Buffer::Directory` → `"directory"`, `Buffer::Content` → `"content"`, `Buffer::Image` → `"image"`, `Buffer::Help` → `"help"`, `Buffer::QuickFix` → `"quickfix"`, `Buffer::Tasks` → `"tasks"`, `Buffer::PathReference` → `"content"`, `Buffer::Empty` → `"empty"`. If no buffer is assigned to the viewport, `buffer_type` SHALL be `nil`.

#### Scenario: Parent subtable has buffer_type set to directory

- **WHEN** `y.hook.on_window_change` is invoked for a Directory window
- **THEN** `ctx.parent.buffer_type` SHALL be `"directory"` since the parent viewport always holds a directory buffer

#### Scenario: Current subtable has buffer_type set to directory

- **WHEN** `y.hook.on_window_change` is invoked for a Directory window
- **THEN** `ctx.current.buffer_type` SHALL be `"directory"` since the current viewport always holds a directory buffer

#### Scenario: Preview subtable has buffer_type set to directory for directory preview

- **WHEN** `y.hook.on_window_change` is invoked and the preview buffer is a `Buffer::Directory` variant
- **THEN** `ctx.preview.buffer_type` SHALL be `"directory"`

#### Scenario: Preview subtable has buffer_type set to content for file preview

- **WHEN** `y.hook.on_window_change` is invoked and the preview buffer is a `Buffer::Content` variant
- **THEN** `ctx.preview.buffer_type` SHALL be `"content"`

#### Scenario: Preview subtable has buffer_type set to image for image preview

- **WHEN** `y.hook.on_window_change` is invoked and the preview buffer is a `Buffer::Image` variant
- **THEN** `ctx.preview.buffer_type` SHALL be `"image"`

#### Scenario: Preview subtable has buffer_type set to empty for empty buffer

- **WHEN** `y.hook.on_window_change` is invoked and the preview buffer is a `Buffer::Empty` variant
- **THEN** `ctx.preview.buffer_type` SHALL be `"empty"`

#### Scenario: Buffer type is nil when no buffer is assigned

- **WHEN** `y.hook.on_window_change` is invoked and a viewport has no buffer in the buffer map
- **THEN** the corresponding subtable's `buffer_type` SHALL be `nil`

#### Scenario: Buffer type property is read-only

- **WHEN** a callback modifies `ctx.preview.buffer_type` to a different value
- **THEN** the system SHALL NOT read back the buffer_type change; buffer types are informational only

### Requirement: on_window_change accepts buffer types for all viewports

The `invoke_on_window_change` function in `yeet-lua` SHALL accept buffer type strings for all three viewports (`[Option<&str>; 3]`) instead of a single `preview_is_directory: bool` parameter. The `invoke_on_window_change_for_focused` helper in `yeet-frontend` SHALL resolve buffer types for all three viewports from the buffer map and pass them to the Lua invocation function.

#### Scenario: invoke_on_window_change receives three buffer types

- **WHEN** a navigation function triggers `invoke_on_window_change_for_focused`
- **THEN** the helper SHALL resolve the buffer type for parent, current, and preview buffers and pass all three as `[Option<&str>; 3]` to `yeet_lua::invoke_on_window_change`

#### Scenario: Buffer type resolved from Buffer enum

- **WHEN** the helper resolves buffer types
- **THEN** it SHALL call `buffer_type_for_lua()` on each `Buffer` instance to get the string representation

## MODIFIED Requirements

### Requirement: on_window_change context table structure

The context table for `on_window_change` SHALL contain the same fields as `on_window_create` for Directory windows: `type`, `parent`, `current`, and `preview` viewport settings tables. Each viewport subtable (`parent`, `current`, `preview`) SHALL include a `path` string field set to the resolved path of that viewport's buffer and a `buffer_type` string field set to the buffer type of that viewport's underlying buffer. The top-level `path` field SHALL NOT be present on the `on_window_change` context table. The top-level `preview_is_directory` field SHALL NOT be present on the `on_window_change` context table.

#### Scenario: Context contains per-viewport paths and buffer types

- **WHEN** `y.hook.on_window_change` is invoked for a Directory window
- **THEN** the context table SHALL have the structure `{ type = "directory", parent = { path = "<parent_dir>", buffer_type = "<type>", <viewport_settings> }, current = { path = "<current_dir>", buffer_type = "<type>", <viewport_settings> }, preview = { path = "<preview_target>", buffer_type = "<type>", <viewport_settings> } }`

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

#### Scenario: Path properties are read-only

- **WHEN** a callback modifies `ctx.current.path` to a different value
- **THEN** the system SHALL NOT read back the path change; viewport paths are informational only

## REMOVED Requirements

### Requirement: on_window_change context table structure — preview_is_directory field

**Reason**: The top-level `preview_is_directory` boolean is replaced by the per-viewport `buffer_type` property. Checking `ctx.preview.buffer_type == "directory"` provides the same functionality in a more general and consistent way.

**Migration**: Replace `ctx.preview_is_directory` with `ctx.preview.buffer_type == "directory"` in all hook callbacks.
