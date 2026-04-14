## MODIFIED Requirements

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
