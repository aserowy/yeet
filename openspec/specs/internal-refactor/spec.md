### Requirement: BufferTheme is created at point of use
Functions that call `yeet_buffer::view()` SHALL create the `BufferTheme` locally via `theme.to_buffer_theme()` or `theme.to_buffer_theme_with_border()` rather than receiving it as a threaded parameter from parent functions. The constructed `BufferTheme` SHALL contain only `Color` values.

#### Scenario: Intermediate layout functions do not accept BufferTheme
- **WHEN** `render_window` is called to lay out split or directory windows
- **THEN** its signature does not include a `BufferTheme` parameter

#### Scenario: Leaf rendering functions create BufferTheme locally
- **WHEN** `render_buffer_slot` or `render_directory_buffer` needs to call `yeet_buffer::view()`
- **THEN** it creates a `BufferTheme` from the `&Theme` parameter and passes it to the view call

### Requirement: from_enumeration accepts Theme reference
The `from_enumeration` function SHALL accept a `&Theme` parameter instead of individual ANSI color strings. It SHALL extract the needed color tokens (`BufferFileFg`, `BufferDirectoryFg`) internally from the theme.

#### Scenario: Directory entry uses theme token
- **WHEN** `from_enumeration` is called with `ContentKind::Directory` and a theme where `BufferDirectoryFg` is `Color::LightBlue`
- **THEN** the returned `BufferLine` content contains the ANSI code for light blue foreground (`\x1b[94m`)

#### Scenario: File entry uses theme token
- **WHEN** `from_enumeration` is called with `ContentKind::File` and a theme where `BufferFileFg` is `Color::White`
- **THEN** the returned `BufferLine` content contains the ANSI code for white foreground (`\x1b[37m`)

#### Scenario: Call sites pass only content, kind, and theme
- **WHEN** callers invoke `from_enumeration`
- **THEN** they pass `(content, kind, theme)` with no pre-computed ANSI strings

### Requirement: Cursor mode codes are buffer-view constants
The buffer view module SHALL define cursor mode codes (`\x1b[7m`, `\x1b[27m`, `\x1b[4m`, `\x1b[24m`) and reset code (`\x1b[0m`) as module-level constants. These SHALL NOT be part of `BufferTheme`.

#### Scenario: Cursor normal mode uses constant
- **WHEN** the buffer view renders a cursor in Normal mode
- **THEN** it uses a module-level constant for the reverse-video code, not a `BufferTheme` field

#### Scenario: Cursor line reset uses constant
- **WHEN** the buffer view renders the cursor line background
- **THEN** it uses a module-level constant for the ANSI reset code, not a `BufferTheme` field
