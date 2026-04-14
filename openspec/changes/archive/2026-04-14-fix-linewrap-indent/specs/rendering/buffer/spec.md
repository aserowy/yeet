## MODIFIED Requirements

### Requirement: Continuation lines have no prefix
Continuation lines (all visual lines after the first for a wrapped BufferLine) SHALL NOT display signs, line numbers, or custom prefix. They SHALL be indented with spaces matching the total offset width (`get_offset_width`) so their content aligns with the first line's content area. The indentation width SHALL equal `get_precontent_width() + get_precontent_border_width()` — computed once via `get_offset_width()`, not by separately adding border width again.

#### Scenario: First line has line number, continuation does not
- **WHEN** a BufferLine wraps into two visual lines and line numbers are enabled
- **THEN** the first visual line SHALL show the line number and the second SHALL show spaces of the same width

#### Scenario: Continuation indent matches first line prefix area
- **WHEN** a BufferLine wraps into two visual lines with signs, line numbers, prefix column, and border active
- **THEN** the continuation line's space indentation width SHALL equal the first line's total prefix area width (signs + line number + prefix column + border)
- **THEN** the first content character of the continuation line SHALL be at the same horizontal position as the first content character of the first visual line
