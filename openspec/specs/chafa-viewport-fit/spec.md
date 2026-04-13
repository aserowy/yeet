### Requirement: Chafa view-size uses content-area dimensions
The system SHALL pass the effective content-area width and height to chafa's `--view-size` parameter, subtracting all viewport offsets (sign columns, line numbers, prefix columns, border) from the raw viewport dimensions before invoking chafa.

#### Scenario: Default preview viewport with no offsets
- **WHEN** the preview viewport has `sign_column_width: 0`, `line_number: None`, `prefix_column_width: 0`, `show_border: false`
- **THEN** chafa SHALL receive `--view-size {viewport.width}x{viewport.height}` unchanged, since content width equals viewport width

#### Scenario: Preview viewport with border enabled (vertical split)
- **WHEN** the preview viewport has `show_border: true` (e.g., inside a vertical split)
- **THEN** chafa SHALL receive a width reduced by the border column (1 column for `Borders::RIGHT`), so `--view-size {viewport.width - 1}x{viewport.height}`

#### Scenario: Preview viewport with sign columns and line numbers
- **WHEN** the preview viewport has non-zero `sign_column_width` or `line_number_width` (e.g., set by a plugin)
- **THEN** chafa SHALL receive a width reduced by the total offset width (sign columns + line numbers + prefix column + border), matching the `get_content_width` calculation

### Requirement: Content-area rect computed at load site
The system SHALL compute the content-area dimensions at the `Action::Load` call site in `action.rs` where the preview viewport context is available, rather than passing raw viewport dimensions to the task.

#### Scenario: LoadPreview receives content-area rect
- **WHEN** `Action::Load` triggers a file preview for a non-directory path
- **THEN** the `Task::LoadPreview` rect SHALL have its width set to the content-area width (viewport width minus offsets) and height set to the viewport height
