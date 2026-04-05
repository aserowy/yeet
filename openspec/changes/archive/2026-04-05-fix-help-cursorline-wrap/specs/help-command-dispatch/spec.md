## ADDED Requirements

### Requirement: Help buffer lines produce exactly one rendered line each
Each line in the help buffer SHALL produce exactly one `ratatui::Line` when rendered. Syntax-highlighted content containing embedded newlines SHALL be split into separate `BufferLine`s so that each produces a single rendered line.

#### Scenario: Cursor line width matches viewport width
- **WHEN** the cursor is on a help buffer line that contains syntax-highlighted markdown
- **THEN** the rendered cursor line width SHALL equal the viewport content width, with no visual line break or wrap

#### Scenario: Highlighted string with trailing newline
- **WHEN** a syntax-highlighted string ends with a trailing newline
- **THEN** it SHALL produce exactly one `BufferLine`, not an additional empty line
