## ADDED Requirements

### Requirement: Shared buffer model supports icon column segment
The `@yeet-buffer` model SHALL define an icon-column prefix segment as part of the common line-prefix structure so all buffer definitions can represent the segment consistently.

### Requirement: Icon-column width defaults by plugin availability
The shared `@yeet-buffer` icon-column segment SHALL default to width `0`.

#### Scenario: Width defaults to zero
- **WHEN** a window is created before any directory-icons hook updates width
- **THEN** `@yeet-buffer` icon-column width remains `0` and no icon cell is reserved

#### Scenario: Plugin hook sets width to one
- **WHEN** `yeet-directory-icons` runs its `on_window_create` hook
- **THEN** `@yeet-buffer` icon-column width is set to `1` for icon rendering

#### Scenario: Non-directory buffer can represent icon segment
- **WHEN** a non-directory buffer line is represented through `@yeet-buffer`
- **THEN** the line-prefix model includes the icon-column segment in its schema/structure even if that buffer type does not populate an icon value

### Requirement: Directory window uses shared buffer icon rendering
The directory window SHALL use shared `@yeet-buffer` icon-column rendering across all three of its buffer instances rather than window-specific icon drawing logic.

#### Scenario: Three directory buffers share icon-column path
- **WHEN** a directory window renders its three `@yeet-buffer` instances
- **THEN** each instance uses the shared `@yeet-buffer` icon-column function/contract for prefix rendering

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
