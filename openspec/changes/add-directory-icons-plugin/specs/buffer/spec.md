## ADDED Requirements

### Requirement: Directory buffer renders a dedicated icon column
Directory buffers SHALL render an icon column between line numbers and filename text for each first visual line of a directory entry. The icon column SHALL have a fixed width and SHALL be treated as prefix content.

#### Scenario: Icon column appears between line number and filename
- **WHEN** a directory buffer line is rendered with line numbers enabled
- **THEN** the rendered prefix order is line number, icon column, then filename text

#### Scenario: Wrapped continuation line omits icon prefix
- **WHEN** a directory entry wraps to continuation visual lines
- **THEN** only the first visual line includes the icon column and continuation lines use prefix padding only

### Requirement: Directory icon column is non-editable
The icon column SHALL NOT be part of editable buffer text. Entering Normal or Insert mode SHALL preserve edit operations on filename content only.

#### Scenario: Insert mode does not modify icon column
- **WHEN** the user enters Insert mode on a directory entry
- **THEN** inserted characters are applied to filename content and the icon column remains unchanged

#### Scenario: Deletion commands skip icon column
- **WHEN** a deletion command targets text at the start of a directory filename
- **THEN** only filename characters are removed and icon glyphs are not deleted

#### Scenario: Buffer text excludes icon column bytes
- **WHEN** a directory entry is rendered with an icon prefix
- **THEN** the underlying buffer text content for that line contains filename text only and no icon-column characters

### Requirement: Cursor origin remains on filename text
When a directory entry is focused, the cursor SHALL start at the first filename character rather than inside the icon column.

#### Scenario: Cursor starts at filename start
- **WHEN** a directory buffer is opened and an entry is focused
- **THEN** the cursor column maps to the first character of the filename text
