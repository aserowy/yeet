## ADDED Requirements

### Requirement: Wrapped continuation lines preserve ANSI styling
When a styled line wraps into multiple visual lines, each continuation segment SHALL carry the ANSI escape sequences that were active at the wrap boundary. The visual styling (colors, bold, etc.) SHALL be continuous across all segments of a wrapped line.

#### Scenario: Red-colored line wraps with color preserved
- **WHEN** a line styled with red foreground wraps into two visual lines
- **THEN** both visual lines SHALL render in red foreground

#### Scenario: Multiple style changes within a wrapped line
- **WHEN** a line contains multiple ANSI style changes and wraps at a point after the second style change
- **THEN** the continuation segment SHALL start with the cumulative styling active at the wrap point
