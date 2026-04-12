## Requirements

### Requirement: Line wrapping

Buffers SHALL support a word wrapping mode that breaks long lines at word boundaries to fit within the viewport width.

When word wrapping is enabled on a viewport, the view layer SHALL:

- Break lines at space characters when possible, falling back to character boundaries
- Preserve ANSI escape sequences across wrapped segments
- Display line numbers and signs only on the first visual line of a wrapped logical line
- Indent continuation lines to align with the content column of the first line
- Adjust cursor positioning to account for wrapped segments
- Adjust vertical scrolling to account for visual line heights

When word wrapping is disabled, horizontal scrolling SHALL be used for lines exceeding viewport width.

Word wrapping MAY be toggled at runtime via the `:set wrap` and `:set nowrap` commands.

#### Scenario: Wrap is disabled by default
- **WHEN** a ViewPort is created with default settings
- **THEN** `wrap` SHALL be `false` and lines SHALL be horizontally scrolled as before

#### Scenario: Wrap is enabled
- **WHEN** `wrap` is set to `true` on a ViewPort
- **THEN** lines longer than the content width SHALL be broken into multiple visual lines

#### Scenario: Wrap toggled at runtime via set command

- **WHEN** a viewport has wrap disabled and the user executes `:set wrap`
- **THEN** the viewport SHALL re-render with word wrapping enabled
- **THEN** horizontal_index SHALL be reset to 0

#### Scenario: Nowrap toggled at runtime via set command

- **WHEN** a viewport has wrap enabled and the user executes `:set nowrap`
- **THEN** the viewport SHALL re-render without word wrapping

### Requirement: Lines wrap at word boundaries
When wrap is enabled, lines SHALL be broken at the last space character that fits within the viewport content width. If a single word is longer than the content width, the line SHALL be broken at exactly the content width (character-count fallback).

#### Scenario: Line breaks at space boundary
- **WHEN** a line contains "hello world foo" and the content width is 10
- **THEN** the line SHALL be rendered as two visual lines: "hello" and "world foo", breaking at the space before "world"

#### Scenario: Word longer than viewport breaks by character
- **WHEN** a line contains a word with no spaces that is longer than the content width
- **THEN** the word SHALL be broken at exactly the content width boundary

#### Scenario: Line fits within viewport
- **WHEN** a line is shorter than or equal to the content width
- **THEN** the line SHALL be rendered as a single visual line with no wrapping

### Requirement: Continuation lines have no prefix
Continuation lines (all visual lines after the first for a wrapped BufferLine) SHALL NOT display signs, line numbers, or custom prefix. They SHALL be indented with spaces matching the prefix width so their content aligns with the first line's content area.

#### Scenario: First line has line number, continuation does not
- **WHEN** a BufferLine wraps into two visual lines and line numbers are enabled
- **THEN** the first visual line SHALL show the line number and the second SHALL show spaces of the same width

### Requirement: Vertical movement skips wrapped lines
The `j` and `k` motions SHALL move between BufferLine indices, not between visual lines within a wrapped BufferLine. `gg` and `G` SHALL move to the first and last BufferLine respectively.

#### Scenario: j on a wrapped line moves to next BufferLine
- **WHEN** the cursor is on a BufferLine that wraps into 3 visual lines and the user presses `j`
- **THEN** the cursor SHALL move to the next BufferLine, not to the second visual line of the current BufferLine

### Requirement: Horizontal movement traverses across wrap boundaries
All horizontal motions SHALL traverse the full BufferLine content as if it were a single unwrapped line. When the cursor crosses a wrap boundary, it SHALL visually move to the next or previous visual line within the same BufferLine. At the BufferLine boundary, motions SHALL stop (existing boundary behavior preserved). The following motions are affected: `h`, `l`, `w`, `W`, `b`, `B`, `e`, `E`, `ge`, `gE`, `f`, `F`, `t`, `T`, `;`, `,`, `0`, `$`, `I`, `A`, `a`, `x`, `s`, `c<motion>`, `d<motion>`, Left Arrow, Right Arrow, Backspace, and Delete.

#### Scenario: l crosses wrap boundary
- **WHEN** the cursor is at the last character of the first visual line of a wrapped BufferLine and the user presses `l`
- **THEN** the cursor SHALL move to the first character of the second visual line (same BufferLine)

#### Scenario: l at end of BufferLine does nothing
- **WHEN** the cursor is at the last character of the last visual line of a wrapped BufferLine and the user presses `l`
- **THEN** the cursor SHALL not move (existing behavior)

#### Scenario: w crosses wrap boundary
- **WHEN** the next word starts on the second visual line of the same BufferLine and the user presses `w`
- **THEN** the cursor SHALL move to the start of that word on the second visual line

#### Scenario: f finds character across wrap boundary
- **WHEN** the target character of `f<char>` is on the second visual line of the same BufferLine
- **THEN** the cursor SHALL move to that character on the second visual line

#### Scenario: 0 moves to start of BufferLine from continuation line
- **WHEN** the cursor is on the second visual line of a wrapped BufferLine and the user presses `0`
- **THEN** the cursor SHALL move to the first character of the first visual line (BufferLine start)

#### Scenario: $ moves to end of BufferLine from first visual line
- **WHEN** the cursor is on the first visual line of a wrapped BufferLine and the user presses `$`
- **THEN** the cursor SHALL move to the last character of the last visual line (BufferLine end)

### Requirement: Cursor line background spans all wrapped lines
When the cursor is on a wrapped BufferLine, the cursor line background color SHALL be applied to all visual lines of that BufferLine, not just the visual line containing the cursor character.

#### Scenario: Cursor on wrapped line highlights all segments
- **WHEN** a BufferLine wraps into 3 visual lines and the cursor is on that BufferLine
- **THEN** all 3 visual lines SHALL have the cursor line background color

### Requirement: Horizontal scrolling is disabled when wrap is enabled
When `wrap` is `true`, `viewport.horizontal_index` SHALL be forced to `0`. The viewport SHALL NOT horizontally scroll wrapped content.

#### Scenario: Horizontal index reset with wrap
- **WHEN** `wrap` is `true` and the viewport would normally need horizontal scrolling
- **THEN** `horizontal_index` SHALL remain `0` and the content SHALL wrap instead

### Requirement: Viewport accounts for wrapped line heights
When wrap is enabled, the viewport SHALL calculate the visual height of each BufferLine (number of visual lines after wrapping) to determine how many BufferLines fit on screen. Vertical scrolling SHALL ensure the cursor's BufferLine is fully visible, including all its wrapped visual lines.

#### Scenario: Tall wrapped line reduces visible BufferLines
- **WHEN** a BufferLine wraps into 5 visual lines and the viewport height is 10
- **THEN** fewer total BufferLines SHALL fit on screen compared to when the same line is not wrapped

#### Scenario: Scroll ensures cursor line is fully visible
- **WHEN** the cursor is on a BufferLine that wraps into 4 visual lines and only 2 of those lines are within the viewport
- **THEN** the viewport SHALL scroll to show all 4 visual lines of the cursor's BufferLine

### Requirement: Wrapped continuation lines preserve ANSI styling
When a styled line wraps into multiple visual lines, each continuation segment SHALL carry the ANSI escape sequences that were active at the wrap boundary. The visual styling (colors, bold, etc.) SHALL be continuous across all segments of a wrapped line.

#### Scenario: Red-colored line wraps with color preserved
- **WHEN** a line styled with red foreground wraps into two visual lines
- **THEN** both visual lines SHALL render in red foreground

#### Scenario: Multiple style changes within a wrapped line
- **WHEN** a line contains multiple ANSI style changes and wraps at a point after the second style change
- **THEN** the continuation segment SHALL start with the cumulative styling active at the wrap point

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
The `on_bufferline_mutate` hook SHALL fire for all buffer types — not just directory buffers. Each hook invocation SHALL provide buffer metadata as a read-only `buffer` object (`ctx.buffer`) on the hook context table. The `buffer` object SHALL contain a `type` field (string matching `BufferType` enum variant names: `"directory"`, `"content"`, `"help"`, `"quickfix"`, `"tasks"`). The `buffer` object SHALL contain a `path` field only for buffer types with an associated path (parent path for directory buffers, file path for content buffers). For buffer types without an associated path (help, quickfix, tasks), the `path` field SHALL be absent (nil). Using a dedicated metadata object ensures the API is extensible — new metadata fields can be added without changing the mutable field surface or breaking existing plugins.

### Requirement: BufferType enum for type-safe invocations
The `invoke_on_bufferline_mutate` function SHALL accept a `BufferType` enum instead of a `&str` for the buffer type parameter. The `BufferType` enum SHALL have variants `Directory`, `Content`, `Help`, `Quickfix`, `Tasks`. Each variant SHALL map to its lowercase string representation (e.g., `BufferType::Directory` → `"directory"`) for injection into the Lua hook context.

#### Scenario: Caller passes enum variant
- **WHEN** a caller invokes `invoke_on_bufferline_mutate` for a directory buffer
- **THEN** it passes `BufferType::Directory` (not the string `"directory"`)

#### Scenario: Enum maps to string in Lua
- **WHEN** the hook fires with `BufferType::Content`
- **THEN** `ctx.buffer.type` in Lua is set to `"content"`

#### Scenario: Invalid buffer type is a compile error
- **WHEN** a caller attempts to pass a misspelled buffer type
- **THEN** it fails to compile because the enum has no matching variant

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
