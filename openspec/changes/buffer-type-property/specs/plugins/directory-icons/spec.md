## MODIFIED Requirements

### Requirement: Plugin sets prefix column width to 2

The `yeet-directory-icons` plugin SHALL set `prefix_column_width` to `2` via the `on_window_create` hook for parent and current directory buffer panes. The plugin SHALL NOT set `prefix_column_width` on the preview viewport in `on_window_create`. Instead, the plugin SHALL register an `on_window_change` callback that conditionally sets `prefix_column_width` to `2` on the preview viewport when the preview buffer type is `"directory"` (checked via `ctx.preview.buffer_type == "directory"`), and sets `prefix_column_width` to `0` when the preview buffer type is not `"directory"`.

#### Scenario: Plugin sets prefix width on parent and current at window create

- **WHEN** `yeet-directory-icons` runs its `on_window_create` hook for a directory window
- **THEN** `prefix_column_width` is set to `2` on parent and current panes only

#### Scenario: Plugin does not set preview prefix width at window create

- **WHEN** `yeet-directory-icons` runs its `on_window_create` hook for a directory window
- **THEN** `prefix_column_width` is NOT set on the preview pane by `on_window_create`

#### Scenario: Plugin sets preview prefix width for directory preview via on_window_change

- **WHEN** the preview target changes to a directory and `on_window_change` fires with `ctx.preview.buffer_type == "directory"`
- **THEN** the plugin sets `ctx.preview.prefix_column_width = 2`

#### Scenario: Plugin clears preview prefix width for file preview via on_window_change

- **WHEN** the preview target changes to a file and `on_window_change` fires with `ctx.preview.buffer_type ~= "directory"`
- **THEN** the plugin sets `ctx.preview.prefix_column_width = 0`

#### Scenario: Non-directory windows are not affected

- **WHEN** `yeet-directory-icons` runs its `on_window_create` hook for a help, quickfix, or tasks window
- **THEN** `prefix_column_width` is not modified by the plugin

#### Scenario: Plugin unavailable keeps zero prefix width

- **WHEN** `yeet-directory-icons` is not installed or not configured
- **THEN** `prefix_column_width` remains at the default `0` and no prefix space is reserved
