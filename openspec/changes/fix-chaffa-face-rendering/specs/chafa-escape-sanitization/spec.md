## ADDED Requirements

### Requirement: Non-SGR escape sequences stripped from chafa output
The system SHALL strip all CSI escape sequences that do not terminate with `m` from chafa's stdout before storing the output as `Preview::Content` lines. Only SGR sequences (color/style codes ending in `m`) SHALL be preserved.

#### Scenario: Chafa output contains cursor movement sequences
- **WHEN** chafa's stdout contains CSI cursor movement sequences (e.g., `\x1b[2C`, `\x1b[1;1H`)
- **THEN** those sequences SHALL be removed from the output lines before they are stored as `Preview::Content`

#### Scenario: Chafa output contains SGR color codes
- **WHEN** chafa's stdout contains SGR sequences (e.g., `\x1b[38;2;255;100;50m`, `\x1b[0m`)
- **THEN** those sequences SHALL be preserved in the output lines

#### Scenario: Chafa output contains erase sequences
- **WHEN** chafa's stdout contains CSI erase sequences (e.g., `\x1b[2J`, `\x1b[K`)
- **THEN** those sequences SHALL be removed from the output lines

### Requirement: Sanitization applied before line splitting
The system SHALL sanitize the chafa stdout as a whole string (or per-line) before the output is split into `Vec<String>` lines and wrapped in `Ansi::new()`, ensuring no non-SGR sequences enter the buffer pipeline.

#### Scenario: Multi-line chafa output sanitized consistently
- **WHEN** chafa produces multi-line output with non-SGR sequences scattered across lines
- **THEN** every line SHALL have non-SGR sequences removed before being stored
