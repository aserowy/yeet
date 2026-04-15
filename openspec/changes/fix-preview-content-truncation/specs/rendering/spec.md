## ADDED Requirements

### Requirement: Syntax highlighting passes full lines to the parser

The syntax highlighting pipeline SHALL pass the full original line content to syntect for highlighting, without any pre-highlight truncation. This ensures syntect's stateful parser maintains correct cross-line state (e.g., knowing when a string literal, comment, or other multi-character token is closed).

#### Scenario: Long URL line is highlighted correctly without style bleed

- **WHEN** a markdown file contains a line `src="https://github.com/user-attachments/assets/4a5268ba-e796-45dc-9ae8-8a41386c0a49"` and the viewport width is 60 characters
- **THEN** syntect SHALL receive the full line including the closing `"` for highlighting
- **THEN** subsequent lines SHALL NOT inherit the string-literal style from the URL line

#### Scenario: Line longer than viewport width preserves parser state

- **WHEN** a file contains a line with 500 characters that includes opening and closing delimiters (e.g., quotes, brackets)
- **THEN** syntect SHALL receive all 500 characters
- **THEN** the parser state after processing that line SHALL correctly reflect that all delimiters are closed

#### Scenario: Multi-line string constructs are highlighted correctly

- **WHEN** a file contains a legitimate multi-line string (e.g., a string literal spanning multiple lines in a programming language)
- **THEN** syntect SHALL maintain correct parser state across the lines
- **THEN** the string-literal styling SHALL apply only to lines that are actually inside the string

### Requirement: Display-width fitting is handled by the view layer

The syntax highlighting function SHALL NOT truncate output to the viewport width. Full-width highlighted content SHALL be stored in the buffer and the existing view-layer mechanisms (horizontal scrolling in nowrap mode, word wrapping in wrap mode) SHALL handle display-width fitting at render time.

#### Scenario: Highlighted content stored at full width

- **WHEN** a file is syntax-highlighted for preview
- **THEN** each `BufferLine` SHALL contain the full highlighted content for the entire source line
- **THEN** the view layer SHALL handle truncation or wrapping to fit the viewport

#### Scenario: Horizontal scrolling works with full-width highlighted lines

- **WHEN** a highlighted line is wider than the viewport and wrap is disabled
- **THEN** the view layer SHALL use `skip_chars` with `horizontal_index` to scroll the visible portion
- **THEN** no content SHALL be lost due to pre-highlight truncation

### Requirement: Non-SGR escape sequence sanitization is preserved

The syntax highlighting pipeline SHALL continue to strip non-SGR escape sequences from highlighted output via `strip_non_sgr_escape_sequences`. This sanitization is the actual security protection against terminal breakout from malicious escape sequences in highlighted content.

#### Scenario: Highlighted output is sanitized

- **WHEN** syntect produces output containing non-SGR CSI sequences (e.g., cursor movement)
- **THEN** those sequences SHALL be stripped before the output is stored as preview content
- **THEN** only SGR sequences (color/style codes ending in `m`) SHALL be preserved
