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

### Requirement: Hook fires for all buffer types with buffer metadata object
The `on_bufferline_mutate` hook SHALL fire for all buffer types — not just directory buffers. Each hook invocation SHALL provide buffer metadata as a read-only `buffer` object (`ctx.buffer`) on the hook context table. The `buffer` object SHALL contain a `type` field (string matching `Buffer` enum variant names: `"directory"`, `"content"`, `"help"`, `"quickfix"`, `"tasks"`). The `buffer` object SHALL contain a `path` field only for buffer types with an associated path (parent path for directory buffers, file path for content buffers). For buffer types without an associated path (help, quickfix, tasks), the `path` field SHALL be absent (nil). Using a dedicated metadata object ensures the API is extensible — new metadata fields can be added without changing the mutable field surface or breaking existing plugins.

#### Scenario: Hook fires for directory buffer entries
- **WHEN** directory content is set or updated via `EnumerationChanged`, `EnumerationFinished`, or `PathsAdded`
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"directory"` and `ctx.buffer.path` set to the parent directory path

#### Scenario: Hook fires for content buffer entries
- **WHEN** a content buffer (file preview) is populated
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"content"` and `ctx.buffer.path` set to the file path

#### Scenario: Hook fires for help buffer entries
- **WHEN** a help buffer is populated
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"help"` and `ctx.buffer.path` absent (nil)

#### Scenario: Hook fires for quickfix buffer entries
- **WHEN** a quickfix buffer is populated
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"quickfix"` and `ctx.buffer.path` absent (nil)

#### Scenario: Hook fires for tasks buffer entries
- **WHEN** a tasks buffer is populated
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"tasks"` and `ctx.buffer.path` absent (nil)

#### Scenario: Buffer metadata object is read-only
- **WHEN** a plugin modifies `ctx.buffer.type` or `ctx.buffer.path` in a hook callback
- **THEN** the changes are NOT read back into the core; the `buffer` object is informational only

#### Scenario: Buffer metadata object is extensible
- **WHEN** a future core version adds additional metadata (e.g., buffer ID)
- **THEN** it can be added as a new field on `ctx.buffer` without changing the existing `type`/`path` contract or breaking existing plugins

### Requirement: Full bufferline is mutable in hook context
Inside the `on_bufferline_mutate` hook, the entire bufferline (excluding line numbers) SHALL be mutable. The mutable fields are: `prefix`, `content` (Ansi string), `search_char_position`, `signs`, and `icon`.

#### Scenario: Plugin mutates icon field
- **WHEN** the hook is invoked for a directory entry
- **THEN** the plugin can set the `icon` field to a glyph string

#### Scenario: Plugin mutates content with ANSI styling
- **WHEN** the hook is invoked for a directory entry
- **THEN** the plugin can prepend ANSI escape sequences to the `content` field to color the text

#### Scenario: Plugin mutates prefix
- **WHEN** the hook is invoked for any buffer entry
- **THEN** the plugin can set or modify the `prefix` field

#### Scenario: Plugin mutates signs
- **WHEN** the hook is invoked for any buffer entry
- **THEN** the plugin can add, remove, or modify entries in the `signs` field

### Requirement: icon_style field removed from BufferLine
The `icon_style` field SHALL be removed from `BufferLine`. The core SHALL NOT apply any icon-related foreground styling to content. All content styling is the plugin's responsibility via direct mutation of the `content` Ansi string.

#### Scenario: No core-applied icon styling
- **WHEN** a bufferline is rendered after hook execution
- **THEN** the core renders the `content` Ansi string as-is without prepending any icon-related styling

#### Scenario: Plugin styles content directly
- **WHEN** the plugin wants to color filename text
- **THEN** it prepends ANSI escape sequences to the `content` field in the hook context

### Requirement: Directory names end with trailing slash
Directory entry names in bufferline content SHALL always end with a trailing slash (`/`) so that users and plugins can differentiate directories from files by name alone.

#### Scenario: Enumerated directory entry has trailing slash
- **WHEN** a directory entry is added to a directory buffer via enumeration
- **THEN** its bufferline content ends with `/`

#### Scenario: File entry has no trailing slash
- **WHEN** a file entry is added to a directory buffer via enumeration
- **THEN** its bufferline content does not end with `/`

#### Scenario: Plugin uses trailing slash for identification
- **WHEN** the hook is invoked for a directory buffer entry
- **THEN** the plugin can determine directory-ness by checking if the content string ends with `/`

### Requirement: ContentKind enum removed
After adopting the trailing-slash naming convention, the `ContentKind` enum SHALL be removed. Directory-ness is encoded in the entry name itself (trailing slash), eliminating the need for a separate type flag.

#### Scenario: No ContentKind in enumeration messages
- **WHEN** enumeration messages are produced by the task runner
- **THEN** entries are strings (with trailing slash for directories) without a separate `ContentKind` discriminant

#### Scenario: No is_directory parameter in hook
- **WHEN** the `on_bufferline_mutate` hook is invoked
- **THEN** there is no `is_directory` field in the context; directory-ness is determined from the content string

### Requirement: Core renders mutated bufferlines without icon styling
The core rendering pipeline SHALL render the `icon` glyph in the icon-column prefix segment and the `content` Ansi string as-is. The core SHALL NOT apply any foreground color to the icon glyph or content text based on icon-related state.

#### Scenario: Icon rendered without core-applied color
- **WHEN** a bufferline has an `icon` set and no `icon_style` exists
- **THEN** the icon glyph is rendered in the icon column using default terminal foreground

#### Scenario: Content rendered as-is
- **WHEN** a bufferline's `content` contains plugin-prepended ANSI sequences
- **THEN** the core renders the content without modification

### Requirement: Directory buffer renders a dedicated icon column
Directory buffers SHALL render an icon column between line numbers and filename text for each first visual line of a directory entry. The icon column SHALL have a fixed width and SHALL be treated as prefix content. The icon content is determined entirely by the plugin's direct mutation of bufferlines via hooks.

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

### Requirement: No fallback for icon column rendering
When no plugin is installed, the icon column remains at width `0` and no icon-related styling is applied. Content renders as plain unstyled text.

#### Scenario: No plugin means no icons and no styling
- **WHEN** `yeet-directory-icons` is not installed
- **THEN** directory entries render as plain filenames with no icons and no foreground color styling
