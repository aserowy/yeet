## REMOVED Requirements

### Requirement: Shared buffer model supports icon column segment
**Reason**: The icon column concept is replaced by the configurable prefix column. Icons are now rendered through the `prefix` field on `BufferLine` with `prefix_column_width` on `ViewPort`.
**Migration**: Use `prefix_column_width` on `ViewPort` and the `prefix` field on `BufferLine` instead of `icon_column_width` and `icon`.

### Requirement: Icon-column width defaults by plugin availability
**Reason**: Replaced by prefix column width which defaults to `0` and is set per-viewport by plugins.
**Migration**: Plugins set `prefix_column_width` via `on_window_create` hook instead of `icon_column_width`.

### Requirement: Directory window uses shared buffer icon rendering
**Reason**: There is no separate icon rendering path. Directory windows use the shared prefix column rendering.
**Migration**: Directory windows render prefix content through the unified prefix column.

### Requirement: Directory buffer renders a dedicated icon column
**Reason**: Replaced by the configurable prefix column. The prefix column handles the same rendering responsibilities.
**Migration**: Use `prefix_column_width` and the `prefix` field on `BufferLine`.

### Requirement: Directory icon column is non-editable
**Reason**: Replaced by prefix column non-editability requirement.
**Migration**: The prefix column inherits the same non-editable behavior.

### Requirement: Cursor origin remains on filename text
**Reason**: Replaced by equivalent prefix column cursor behavior requirement.
**Migration**: Cursor behavior is preserved — cursor starts at content after the prefix column.

### Requirement: No fallback for icon column rendering
**Reason**: Replaced by prefix column fallback behavior.
**Migration**: When `prefix_column_width` is `0`, no prefix space is reserved.

### Requirement: Core renders mutated bufferlines without icon styling
**Reason**: Replaced by prefix-based rendering requirement. The core renders prefix content as-is without applying styling.
**Migration**: Use the `prefix` field for styled content; the core renders it as-is.

### Requirement: icon_style field removed from BufferLine
**Reason**: Already removed. The `icon` field itself is now also removed.
**Migration**: No action needed; styling is done via ANSI sequences in the `prefix` field.

### Requirement: Full bufferline is mutable in hook context
**Reason**: Modified to reflect removal of `icon` field from mutable fields.
**Migration**: See MODIFIED requirement below.

## MODIFIED Requirements

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

## ADDED Requirements

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
When `prefix_column_width` is `0`, no prefix space SHALL be reserved in the viewport layout. The rendering SHALL behave identically to the current behavior when no icon column is configured.

#### Scenario: Zero width means no prefix overhead
- **WHEN** `prefix_column_width` is `0` and the bufferline has no prefix
- **THEN** the content area occupies the full available width after signs, line numbers, and border

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
