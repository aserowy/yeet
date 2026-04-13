## ADDED Requirements

### Requirement: Line number column has consistent width across all lines
The line number column SHALL produce the same number of visible characters for every line in the buffer, regardless of whether the line is the cursor line or not, and regardless of whether the line number mode is Absolute or Relative. The visible character count SHALL equal the configured `line_number_width` for all lines.

#### Scenario: Absolute mode cursor line matches non-cursor line width
- **WHEN** line numbers are displayed in Absolute mode
- **THEN** the cursor line's line number column SHALL produce exactly `line_number_width` visible characters, matching the non-cursor lines

#### Scenario: Relative mode cursor line matches non-cursor line width
- **WHEN** line numbers are displayed in Relative mode
- **THEN** the cursor line's line number column SHALL produce exactly `line_number_width` visible characters, matching the non-cursor lines

#### Scenario: Content column starts at the same position for all lines
- **WHEN** a buffer has line numbers enabled in any mode (Absolute or Relative)
- **THEN** the first character of content (e.g., filename) SHALL start at the same horizontal column for every visible line
