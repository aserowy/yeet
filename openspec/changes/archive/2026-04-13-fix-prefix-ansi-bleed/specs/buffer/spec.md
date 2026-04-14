## ADDED Requirements

### Requirement: Prefix component ANSI styles are isolated

Each prefix component (signs, line numbers, prefix column, border) SHALL render with self-contained ANSI styling so that escape codes from one component do not affect the rendering of subsequent components.

Specifically, when the prefix column renders an icon character, the terminal MUST NOT receive residual ANSI attributes (such as bold, italic, or foreground color) from the preceding sign or line number components.

#### Scenario: Prefix icon renders consistently regardless of line number mode

- **WHEN** a directory viewport has `line_number: Relative` (which emits ANSI color codes) and `prefix_column_width > 0` with a nerdfont icon
- **THEN** the rendered `Span` containing the prefix icon SHALL NOT inherit bold, foreground color, or other style attributes from the line number spans

#### Scenario: Prefix icon renders consistently on cursor line with bold line number

- **WHEN** the cursor line has a bold line number (`\x1b[1m`) followed by a prefix column containing a PUA icon character
- **THEN** the `ansi_to_tui` parsed spans for the prefix column SHALL have a clean style (no inherited bold) and the icon SHALL be rendered in a span that does not carry attributes from the line number span
