## ADDED Requirements

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
