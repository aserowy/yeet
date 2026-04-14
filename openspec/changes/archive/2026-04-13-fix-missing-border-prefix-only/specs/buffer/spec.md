## MODIFIED Requirements

### Requirement: No prefix column when width is zero
When `prefix_column_width` is `0`, no prefix space SHALL be reserved in the viewport layout. The rendering SHALL behave identically to the current behavior when no icon column is configured. The border space between pre-content columns and the content area SHALL NOT be rendered when no pre-content column is active (i.e., `sign_column_width == 0` AND `line_number == None` AND `prefix_column_width == 0`).

#### Scenario: Zero width means no prefix overhead
- **WHEN** `prefix_column_width` is `0` and the bufferline has no prefix
- **THEN** the content area occupies the full available width after signs, line numbers, and border

#### Scenario: Border not rendered when all prefix columns are zero
- **WHEN** `sign_column_width == 0` AND `line_number == None` AND `prefix_column_width == 0`
- **THEN** `get_border_width()` SHALL return `0`

## ADDED Requirements

### Requirement: Border rendered when any pre-content column is active
The 1-cell border space SHALL be rendered whenever any pre-content column is active. This includes when only `prefix_column_width > 0` (with no signs or line numbers). The border condition SHALL check all pre-content column widths: `sign_column_width`, `line_number_width`, and `prefix_column_width`.

#### Scenario: Border with only prefix column active
- **WHEN** `prefix_column_width > 0` AND `sign_column_width == 0` AND `line_number == None`
- **THEN** `get_border_width()` SHALL return `1`
- **THEN** the layout SHALL be `[border][prefix_column][content]`

#### Scenario: Border with signs and prefix column active
- **WHEN** `sign_column_width > 0` AND `prefix_column_width > 0`
- **THEN** `get_border_width()` SHALL return `1`
- **THEN** the layout SHALL be `[signs][line_number][border][prefix_column][content]`

#### Scenario: Content width reduced by border when prefix column only
- **WHEN** `prefix_column_width == 2` AND `sign_column_width == 0` AND `line_number == None` AND `width == 80`
- **THEN** `get_content_width()` SHALL return `80 - 0 (prefix) - 1 (border) - 2 (prefix_column) - 1 (border from show_border) = 76` for a viewport with `show_border == true`
- **THEN** `get_content_width()` SHALL return `80 - 0 (prefix) - 1 (border) - 2 (prefix_column) = 77` for a viewport with `show_border == false`
