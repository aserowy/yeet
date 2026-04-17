## ADDED Requirements

### Requirement: Non-SGR escape sequences stripped from syntax highlight output

The system SHALL strip all CSI escape sequences that do not terminate with `m` from syntax-highlighted preview content before storing the output as `Preview::Content` lines. Only SGR sequences (color/style codes ending in `m`) SHALL be preserved. The same `strip_non_sgr_escape_sequences` function used for chafa output SHALL be reused.

#### Scenario: Highlighted output contains only SGR sequences

- **WHEN** syntect produces highlighted output containing only SGR sequences (e.g., `\x1b[38;2;255;100;50m`, `\x1b[0m`)
- **THEN** those sequences SHALL be preserved in the output lines unchanged

#### Scenario: Highlighted output contains non-SGR CSI sequences

- **WHEN** syntect produces highlighted output containing non-SGR CSI sequences (e.g., cursor movement `\x1b[2C`, erase `\x1b[2J`)
- **THEN** those sequences SHALL be removed from the output lines before they are stored as `Preview::Content`

#### Scenario: Sanitization applied to every highlighted line

- **WHEN** a multi-line file is syntax-highlighted for preview
- **THEN** every line of the highlighted output SHALL have non-SGR sequences removed before being stored

### Requirement: Long lines truncated before syntax highlighting

The system SHALL truncate lines that exceed the preview viewport's content-area width before passing them to syntect for highlighting. Truncation SHALL occur at the character level, cutting the line at the content width boundary. The `highlight()` function SHALL accept a `content_width` parameter representing the maximum visible character count.

#### Scenario: Line shorter than content width

- **WHEN** a source line has fewer characters than the content width
- **THEN** the line SHALL be passed to the highlighter without modification

#### Scenario: Line exceeding content width

- **WHEN** a source line has more characters than the content width (e.g., a 517K-character base64 line in an SVG)
- **THEN** the line SHALL be truncated to the content width before highlighting
- **THEN** the highlighted output SHALL contain at most content-width visible characters

#### Scenario: Content width derived from preview viewport

- **WHEN** a `Task::LoadPreview` is dispatched for a non-image file
- **THEN** the content width passed to the highlight function SHALL be derived from the preview viewport's content-area dimensions (viewport width minus offsets)

### Requirement: Shared sanitization module

The `strip_non_sgr_escape_sequences` function SHALL reside in a shared module (`yeet-frontend/src/task/sanitize.rs`) accessible to both the chafa image preview path and the syntax highlight preview path. Both paths SHALL use the same function instance.

#### Scenario: Chafa output uses shared sanitization

- **WHEN** chafa produces output for image preview
- **THEN** the output SHALL be sanitized using the shared `strip_non_sgr_escape_sequences` from the sanitize module

#### Scenario: Syntax highlight output uses shared sanitization

- **WHEN** syntect produces highlighted output for text preview
- **THEN** the output SHALL be sanitized using the shared `strip_non_sgr_escape_sequences` from the sanitize module

#### Scenario: Existing sanitization tests still pass

- **WHEN** tests for `strip_non_sgr_escape_sequences` are run
- **THEN** all existing test cases (SGR preservation, cursor movement stripping, erase stripping, mixed sequences, plain text, empty string) SHALL pass in the new module location
