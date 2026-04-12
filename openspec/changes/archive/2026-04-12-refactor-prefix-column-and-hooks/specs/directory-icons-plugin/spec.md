## MODIFIED Requirements

### Requirement: Plugin directly mutates bufferlines via hooks
The plugin SHALL implement hook handlers that are invoked for each bufferline across all buffer types. Each hook call receives the complete bufferline fields and a read-only `buffer` metadata object (`ctx.buffer`) with `type` and optionally `path` fields. The plugin directly mutates the bufferline fields in-place. There is no request/response pattern. The plugin SHALL set icons via the `prefix` field instead of the removed `icon` field.

#### Scenario: Plugin receives full bufferline context
- **WHEN** the core invokes the hook for a bufferline
- **THEN** the hook provides mutable access to `prefix`, `content`, `search_char_position`, and `signs`, plus a read-only `ctx.buffer` metadata object with `type` and optionally `path` fields

#### Scenario: Plugin sets icon glyph via prefix for a recognized file
- **WHEN** the plugin's hook handler receives a bufferline for a file with extension `.rs` in a directory buffer
- **THEN** the plugin sets the rust icon glyph with ANSI color in the `prefix` field and prepends rust color ANSI sequence to `content`

#### Scenario: Plugin sets fallback icon via prefix for unrecognized file
- **WHEN** the plugin's hook handler receives a bufferline for a file named `README.unknownext`
- **THEN** the plugin sets the default file icon glyph in the `prefix` field and prepends default color to content

#### Scenario: Plugin sets directory-specific icon via prefix
- **WHEN** the plugin's hook handler receives a bufferline for a directory entry named `.git/`
- **THEN** the plugin sets the git directory icon glyph in the `prefix` field and applies directory color

#### Scenario: Plugin replaces existing prefix on re-processing
- **WHEN** a bufferline already has a prefix set and the hook is invoked again (e.g., during `EnumerationFinished` after `EnumerationChanged`)
- **THEN** the plugin replaces the existing prefix with the newly resolved icon

### Requirement: Plugin styles content by mutating the Ansi string
The plugin SHALL apply foreground color to filename text by prepending ANSI escape sequences to the `content` field in the hook context. The plugin SHALL style the icon glyph by including the ANSI color in the `prefix` field value. There is no separate `icon` or `icon_style` field.

#### Scenario: Plugin prepends ANSI color to content
- **WHEN** the plugin resolves a color for a file entry
- **THEN** it prepends the ANSI foreground escape sequence to the `content` string

#### Scenario: Plugin includes color in prefix string
- **WHEN** the plugin sets an icon glyph via prefix
- **THEN** the prefix string includes the ANSI foreground color prefix and a reset suffix so the icon renders in color

## ADDED Requirements

### Requirement: Plugin sets prefix column width to 2
The `yeet-directory-icons` plugin SHALL set `prefix_column_width` to `2` via the `on_window_create` hook for all directory buffer panes (parent, current, preview). Nerd Font icons typically occupy two terminal cells, so the prefix column must be wide enough to display them.

#### Scenario: Plugin sets prefix width on window create
- **WHEN** `yeet-directory-icons` runs its `on_window_create` hook for a directory window
- **THEN** `prefix_column_width` is set to `2` on all three panes (parent, current, preview)

#### Scenario: Non-directory windows are not affected
- **WHEN** `yeet-directory-icons` runs its `on_window_create` hook for a help, quickfix, or tasks window
- **THEN** `prefix_column_width` is not modified by the plugin

#### Scenario: Plugin unavailable keeps zero prefix width
- **WHEN** `yeet-directory-icons` is not installed or not configured
- **THEN** `prefix_column_width` remains at the default `0` and no prefix space is reserved

## REMOVED Requirements

### Requirement: Plugin owns all icon identification and text color logic
**Reason**: Modified to reflect prefix-based approach. See MODIFIED requirement for updated behavior.
**Migration**: Plugin continues to own icon/color logic but writes to `prefix` instead of `icon`.
