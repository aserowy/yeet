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
Continuation lines (all visual lines after the first for a wrapped BufferLine) SHALL NOT display signs, line numbers, or custom prefix. They SHALL be indented with spaces matching the total offset width (`get_offset_width`) so their content aligns with the first line's content area. The indentation width SHALL equal `get_precontent_width() + get_precontent_border_width()` — computed once via `get_offset_width()`, not by separately adding border width again.

#### Scenario: First line has line number, continuation does not
- **WHEN** a BufferLine wraps into two visual lines and line numbers are enabled
- **THEN** the first visual line SHALL show the line number and the second SHALL show spaces of the same width

#### Scenario: Continuation indent matches first line prefix area
- **WHEN** a BufferLine wraps into two visual lines with signs, line numbers, prefix column, and border active
- **THEN** the continuation line's space indentation width SHALL equal the first line's total prefix area width (signs + line number + prefix column + border)
- **THEN** the first content character of the continuation line SHALL be at the same horizontal position as the first content character of the first visual line

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
Inside the `on_bufferline_mutate` hook, the entire bufferline (excluding line numbers) SHALL be mutable. The mutable fields are: `prefix`, `content` (Ansi string), `search_char_position`, and `signs`.

#### Scenario: Plugin mutates content with ANSI styling
- **WHEN** the hook is invoked for a directory entry
- **THEN** the plugin can prepend ANSI escape sequences to the `content` field to color the text

#### Scenario: Plugin mutates prefix
- **WHEN** the hook is invoked for any buffer entry
- **THEN** the plugin can set or modify the `prefix` field (including icon glyphs with ANSI color sequences)

#### Scenario: Plugin mutates signs
- **WHEN** the hook is invoked for any buffer entry
- **THEN** the plugin can add, remove, or modify entries in the `signs` field

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

### Requirement: Configurable prefix column width on ViewPort
The `ViewPort` struct SHALL have a `prefix_column_width` field (usize) that reserves a fixed number of terminal cells for the prefix column in the buffer rendering. The prefix column width SHALL default to `0` for all viewports.

#### Scenario: Default prefix column width is zero
- **WHEN** a `ViewPort` is created with default settings
- **THEN** `prefix_column_width` SHALL be `0` and no prefix space is reserved

#### Scenario: Plugin sets prefix column width via on_window_create
- **WHEN** a plugin's `on_window_create` hook sets `prefix_column_width` to `2`
- **THEN** the viewport SHALL reserve 2 cells for the prefix column

#### Scenario: Prefix column width is per-viewport
- **WHEN** different viewports have different `prefix_column_width` values
- **THEN** each viewport renders its own prefix column width independently

### Requirement: Commandline buffer prefix column width defaults to 1
The commandline buffer's viewport SHALL have `prefix_column_width` set to `1` to accommodate command count display.

#### Scenario: Commandline prefix width is 1
- **WHEN** a commandline buffer viewport is created
- **THEN** `prefix_column_width` SHALL be `1`

#### Scenario: Other buffers remain at 0
- **WHEN** a non-commandline buffer viewport is created without plugin hooks
- **THEN** `prefix_column_width` SHALL remain at the default `0`

### Requirement: Prefix text is right-aligned within prefix column
When rendering the prefix column, the prefix text SHALL be right-aligned within the allocated `prefix_column_width`. If the prefix content is narrower than the column width, the remaining space SHALL be padded with spaces on the left.

#### Scenario: Single-cell icon in two-cell prefix column
- **WHEN** a bufferline has a 1-cell-wide prefix in a viewport with `prefix_column_width` of `2`
- **THEN** the rendered prefix column SHALL be ` X` (space then icon), right-aligned

#### Scenario: Prefix fills entire column
- **WHEN** a bufferline has a 2-cell-wide prefix in a viewport with `prefix_column_width` of `2`
- **THEN** the rendered prefix column SHALL show the full prefix with no padding

#### Scenario: Empty prefix in non-zero column
- **WHEN** a bufferline has no prefix (None) and `prefix_column_width` is `2`
- **THEN** the rendered prefix column SHALL be two spaces

### Requirement: Prefix column is non-editable
The prefix column SHALL NOT be part of editable buffer text. Entering Normal or Insert mode SHALL preserve edit operations on content text only. The cursor SHALL start at the first content character, not inside the prefix column.

#### Scenario: Insert mode does not modify prefix column
- **WHEN** the user enters Insert mode on a buffer entry with a prefix
- **THEN** inserted characters are applied to content text and the prefix column remains unchanged

#### Scenario: Cursor starts at content start
- **WHEN** a buffer entry is focused and has a non-empty prefix column
- **THEN** the cursor column maps to the first character of the content text

### Requirement: No prefix column when width is zero
When `prefix_column_width` is `0`, no prefix space SHALL be reserved in the viewport layout. The rendering SHALL behave identically to the current behavior when no icon column is configured. The border space between pre-content columns and the content area SHALL NOT be rendered when no pre-content column is active (i.e., `sign_column_width == 0` AND `line_number == None` AND `prefix_column_width == 0`).

#### Scenario: Zero width means no prefix overhead
- **WHEN** `prefix_column_width` is `0` and the bufferline has no prefix
- **THEN** the content area occupies the full available width after signs, line numbers, and border

#### Scenario: Border not rendered when all prefix columns are zero
- **WHEN** `sign_column_width == 0` AND `line_number == None` AND `prefix_column_width == 0`
- **THEN** `get_precontent_border_width()` SHALL return `0`

### Requirement: All bufferline mutate hooks fire after signs are added
The `on_bufferline_mutate` hook SHALL fire AFTER all sign operations (mark signs, quickfix signs) have been applied to the bufferline. This ensures plugins see the complete bufferline state including signs when the hook fires.

#### Scenario: Hook sees signs in enumeration
- **WHEN** directory content is set via `EnumerationChanged`, `EnumerationFinished`, or `PathsAdded`
- **THEN** the `on_bufferline_mutate` hook fires after `set_sign_if_marked` and `set_sign_if_qfix` have been called on the bufferline

#### Scenario: Plugin can read signs in hook
- **WHEN** a bufferline has a mark sign applied before the hook fires
- **THEN** the plugin's hook callback can see the sign in the `signs` field

#### Scenario: Hook fires after signs in all buffer types
- **WHEN** bufferlines are created for any buffer type (directory, content, help, quickfix, tasks)
- **THEN** all sign operations complete before the `on_bufferline_mutate` hook is invoked

### Requirement: Wrapped continuation line omits prefix column
Continuation lines (all visual lines after the first for a wrapped BufferLine) SHALL NOT display the prefix column content. They SHALL be indented with spaces matching the prefix column width so their content aligns with the first line's content area.

#### Scenario: First line has prefix, continuation does not
- **WHEN** a BufferLine wraps into two visual lines and has a prefix set
- **THEN** the first visual line SHALL show the prefix and the second SHALL show spaces of the same width

### Requirement: Line number column has consistent width across all lines
The line number column SHALL produce the same number of visible characters for every line in the buffer, regardless of whether the line is the cursor line or not, and regardless of whether the line number mode is Absolute or Relative. The visible character count SHALL equal the configured `line_number_width` for all lines.

#### Scenario: Absolute mode cursor line matches non-cursor line width
- **WHEN** line numbers are displayed in Absolute mode
- **THEN** the cursor line's line number column SHALL produce exactly `line_number_width` visible characters, matching the non-cursor lines

#### Scenario: Relative mode cursor line matches non-cursor line width
- **WHEN** line numbers are displayed in Relative mode
- **THEN** the cursor line's line number column SHALL produce exactly `line_number_width` visible characters, matching the non-cursor lines

#### Scenario: Content column starts at the same position for all lines
- **WHEN** a buffer has line numbers enabled in any mode (Absolute or Relative)
- **THEN** the first character of content (e.g., filename) SHALL start at the same horizontal column for every visible line

### Requirement: Prefix component ANSI styles are isolated

Each prefix component (signs, line numbers, prefix column, border) SHALL render with self-contained ANSI styling so that escape codes from one component do not affect the rendering of subsequent components.

Specifically, when the prefix column renders an icon character, the terminal MUST NOT receive residual ANSI attributes (such as bold, italic, or foreground color) from the preceding sign or line number components.

#### Scenario: Prefix icon renders consistently regardless of line number mode

- **WHEN** a directory viewport has `line_number: Relative` (which emits ANSI color codes) and `prefix_column_width > 0` with a nerdfont icon
- **THEN** the rendered `Span` containing the prefix icon SHALL NOT inherit bold, foreground color, or other style attributes from the line number spans

#### Scenario: Prefix icon renders consistently on cursor line with bold line number

- **WHEN** the cursor line has a bold line number (`\x1b[1m`) followed by a prefix column containing a PUA icon character
- **THEN** the `ansi_to_tui` parsed spans for the prefix column SHALL have a clean style (no inherited bold) and the icon SHALL be rendered in a span that does not carry attributes from the line number span

### Requirement: Border rendered when any pre-content column is active
The 1-cell border space SHALL be rendered whenever any pre-content column is active. This includes when only `prefix_column_width > 0` (with no signs or line numbers). The border condition SHALL check all pre-content column widths: `sign_column_width`, `line_number_width`, and `prefix_column_width`.

#### Scenario: Border with only prefix column active
- **WHEN** `prefix_column_width > 0` AND `sign_column_width == 0` AND `line_number == None`
- **THEN** `get_precontent_border_width()` SHALL return `1`
- **THEN** the layout SHALL be `[border][prefix_column][content]`

#### Scenario: Border with signs and prefix column active
- **WHEN** `sign_column_width > 0` AND `prefix_column_width > 0`
- **THEN** `get_precontent_border_width()` SHALL return `1`
- **THEN** the layout SHALL be `[signs][line_number][border][prefix_column][content]`

#### Scenario: Content width reduced by border when prefix column only
- **WHEN** `prefix_column_width == 2` AND `sign_column_width == 0` AND `line_number == None` AND `width == 80`
- **THEN** `get_content_width()` SHALL return `80 - 0 (precontent) - 1 (border) - 2 (prefix_column) - 1 (border from show_border) = 76` for a viewport with `show_border == true`
- **THEN** `get_content_width()` SHALL return `80 - 0 (precontent) - 1 (border) - 2 (prefix_column) = 77` for a viewport with `show_border == false`

### Requirement: Optional precontent border width override
The `ViewPort` struct SHALL have an optional `precontent_border_width` field (`Option<usize>`) that, when set to `Some(n)`, overrides the computed precontent border width with `n`. When `None` (the default), the existing behavior SHALL be preserved: border width is 1 when `get_precontent_width() > 0`, and 0 otherwise.

#### Scenario: Default is None (computed behavior)
- **WHEN** a `ViewPort` is created with default settings
- **THEN** `precontent_border_width` SHALL be `None` and `get_precontent_border_width()` SHALL return the computed value

#### Scenario: Override to zero suppresses border
- **WHEN** `precontent_border_width` is `Some(0)` and `get_precontent_width() > 0`
- **THEN** `get_precontent_border_width()` SHALL return `0`

#### Scenario: Override enforces width regardless of precontent
- **WHEN** `precontent_border_width` is `Some(2)` and `get_precontent_width() == 0`
- **THEN** `get_precontent_border_width()` SHALL return `2`

### Requirement: Commandline viewport has no precontent border
The commandline buffer's viewport SHALL have `precontent_border_width` set to `Some(0)` to suppress the border space, ensuring the prefix column (used for command count) is rendered without an additional separator cell.

#### Scenario: Commandline viewport border is zero
- **WHEN** a commandline buffer viewport is created with default settings
- **THEN** `precontent_border_width` SHALL be `Some(0)` and `get_precontent_border_width()` SHALL return `0`
