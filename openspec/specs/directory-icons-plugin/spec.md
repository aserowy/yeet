## Purpose

The `yeet-directory-icons` plugin provides icon glyphs and color styling for directory buffer entries. It resolves icons by file extension, filename, or directory name using Nerd Font defaults, and applies foreground colors to both icon glyphs and filename text via direct bufferline mutation hooks.

## Requirements

### Requirement: Plugin owns all icon identification and text color logic
The `yeet-directory-icons` plugin SHALL contain all logic for determining which icon glyph to display and how to color both the icon glyph and the filename text. The core SHALL NOT contain any icon resolution tables, extension mappings, or color rules; it only invokes hooks and the plugin directly mutates bufferlines.

### Requirement: Plugin checks buffer type before acting
The plugin SHALL check the buffer type via the `ctx.buffer.type` field (from the read-only `buffer` metadata object, populated from the `BufferType` enum's string representation) provided in each `on_bufferline_mutate` hook invocation and only mutate bufferlines for file/directory-related buffer types (e.g., `"directory"` type). The plugin SHALL skip non-file buffer types (e.g., `"help"`, `"quickfix"`, `"tasks"`).

#### Scenario: Plugin processes directory buffer entries
- **WHEN** the hook fires with `ctx.buffer.type` equal to `"directory"`
- **THEN** the plugin resolves the icon and colors for the entry and mutates the bufferline

#### Scenario: Plugin skips help buffer entries
- **WHEN** the hook fires with `ctx.buffer.type` equal to `"help"`
- **THEN** the plugin does not mutate the bufferline

#### Scenario: Plugin skips quickfix buffer entries
- **WHEN** the hook fires with `ctx.buffer.type` equal to `"quickfix"`
- **THEN** the plugin does not mutate the bufferline

#### Scenario: Plugin skips tasks buffer entries
- **WHEN** the hook fires with `ctx.buffer.type` equal to `"tasks"`
- **THEN** the plugin does not mutate the bufferline

### Requirement: Plugin uses trailing slash to detect directories
The plugin SHALL determine whether an entry is a directory by checking if the content string ends with a trailing slash (`/`). No separate `is_directory` flag is needed.

#### Scenario: Plugin identifies directory by trailing slash
- **WHEN** the hook receives a bufferline whose content ends with `/`
- **THEN** the plugin treats the entry as a directory and uses directory icon resolution

#### Scenario: Plugin identifies file by absence of trailing slash
- **WHEN** the hook receives a bufferline whose content does not end with `/`
- **THEN** the plugin treats the entry as a file and uses file icon resolution

### Requirement: Plugin styles content by mutating the Ansi string
The plugin SHALL apply foreground color to filename text by prepending ANSI escape sequences to the `content` field in the hook context. The plugin SHALL also style the icon glyph by including the ANSI color in the `icon` field value. There is no separate `icon_style` field.

#### Scenario: Plugin prepends ANSI color to content
- **WHEN** the plugin resolves a color for a file entry
- **THEN** it prepends the ANSI foreground escape sequence to the `content` string

#### Scenario: Plugin includes color in icon string
- **WHEN** the plugin sets an icon glyph
- **THEN** the icon string includes the ANSI foreground color prefix and a reset suffix so the icon renders in color

### Requirement: Plugin directly mutates bufferlines via hooks
The plugin SHALL implement hook handlers that are invoked for each bufferline across all buffer types. Each hook call receives the complete bufferline fields and a read-only `buffer` metadata object (`ctx.buffer`) with `type` and optionally `path` fields. The plugin directly mutates the bufferline fields in-place. There is no request/response pattern.

#### Scenario: Plugin receives full bufferline context
- **WHEN** the core invokes the hook for a bufferline
- **THEN** the hook provides mutable access to `prefix`, `content`, `search_char_position`, `signs`, and `icon`, plus a read-only `ctx.buffer` metadata object with `type` and optionally `path` fields

#### Scenario: Plugin sets icon glyph for a recognized file
- **WHEN** the plugin's hook handler receives a bufferline for a file with extension `.rs` in a directory buffer
- **THEN** the plugin sets the rust icon glyph in `icon` and prepends rust color ANSI sequence to `content`

#### Scenario: Plugin sets fallback icon for unrecognized file
- **WHEN** the plugin's hook handler receives a bufferline for a file named `README.unknownext`
- **THEN** the plugin sets the default file icon glyph and prepends default color to content

#### Scenario: Plugin sets directory-specific icon
- **WHEN** the plugin's hook handler receives a bufferline for a directory entry named `.git/`
- **THEN** the plugin sets the git directory icon glyph and applies directory color (using a directory-specific token distinct from the file default)

#### Scenario: Plugin replaces existing icon on re-processing
- **WHEN** a bufferline already has an icon set and the hook is invoked again (e.g., during `EnumerationFinished` after `EnumerationChanged`)
- **THEN** the plugin replaces the existing icon with the newly resolved icon

### Requirement: Rule mapping applies to all matching entries
Icon/class/color resolution SHALL be rule-based by extension, exact filename, or directory name, and each rule SHALL apply uniformly to every matching entry. All rule logic lives in the plugin.

### Requirement: One unified mapping configuration in the plugin
The directory-icons plugin SHALL use a single, easy-to-extend mapping configuration that stores both file rules (extension/name) and directory-name rules.

#### Scenario: File and directory rules share one mapping source
- **WHEN** the plugin loads its mapping configuration
- **THEN** file extension/name rules and directory-name rules are read from the same configuration structure

#### Scenario: New rule can be added without split config updates
- **WHEN** a user adds a new extension or directory-name mapping rule
- **THEN** only one mapping list/source needs to be updated for icon/class/color behavior to take effect

#### Scenario: Extension rule applies to all matching files
- **WHEN** the plugin's hook handler processes multiple file entries matching the `*.rs` extension rule
- **THEN** each matching entry's bufferline is mutated to the rust icon/class mapping and associated default base color

#### Scenario: Named default directories use configured mapping
- **WHEN** a directory entry name is one of `.direnv/`, `target/`, `.git/`, or `.github/`
- **THEN** the plugin's icon/class/color mutation uses the configured default directory-name mapping for that entry name

#### Scenario: Known Nerd Font file icon defaults are preseeded
- **WHEN** a file entry has a filename/extension with a corresponding Nerd Font icon in the default set
- **THEN** the plugin's icon/class/color mutation uses the preseeded default mapping for that file entry

### Requirement: Plugin registers DirectoryIconsColor theme tokens
The plugin SHALL register a `DirectoryIconsColor*` theme token for every unique color in its rule set. Token names SHALL follow the pattern `DirectoryIconsColor<Identifier>` where `<Identifier>` is derived from the extension, filename, or directory name (e.g., `DirectoryIconsColorRs`, `DirectoryIconsColorTxt`, `DirectoryIconsColorMakefile`, `DirectoryIconsColorDotEnv`, `DirectoryIconsColorGoMod`, `DirectoryIconsColorDefaultDirectory`, `DirectoryIconsColorDefaultFile`). During `setup()`, the plugin SHALL set each token's default value in `y.theme` ONLY if the token is not already set. During bufferline mutation, the plugin SHALL resolve colors by reading these tokens from `y.theme`, not from hardcoded hex values. This makes every icon/text color user-overrideable.

#### Scenario: Plugin sets token defaults during setup
- **WHEN** `yeet-directory-icons` runs `setup()` and a `DirectoryIconsColor*` token is not yet set in `y.theme`
- **THEN** the plugin sets the token to the built-in Nerd Font default hex color

#### Scenario: Plugin does not overwrite existing token
- **WHEN** `yeet-directory-icons` runs `setup()` and a `DirectoryIconsColor*` token is already set in `y.theme` by a theme plugin
- **THEN** the plugin leaves the existing value unchanged

#### Scenario: Plugin uses token value during mutation
- **WHEN** the plugin mutates a bufferline for a `*.rs` file
- **THEN** it reads the color from `y.theme.DirectoryIconsColorRs` and uses that color for ANSI styling

#### Scenario: All extension/filename/directory color tokens are registered
- **WHEN** `setup()` completes
- **THEN** every unique color mapping in `ext_map`, `name_map`, and `dir_map` has a corresponding `DirectoryIconsColor*` token in `y.theme`

#### Scenario: Directory entries use a separate token from file entries
- **WHEN** the plugin mutates a directory entry's bufferline
- **THEN** the applied color token is a directory-specific token (e.g., `DirectoryIconsColorDefaultDirectory`), distinct from the default file icon token (`DirectoryIconsColorDefaultFile`)

### Requirement: Plugin respects existing theme token values
When a theme plugin (e.g., `yeet-bluloco-theme`) sets `DirectoryIconsColor*` theme tokens before `yeet-directory-icons` runs `setup()`, the directory-icons plugin SHALL check for existing theme values and NOT overwrite them. The plugin only sets default values for tokens that have not been set by a theme plugin.

#### Scenario: Theme plugin sets token first
- **WHEN** `yeet-bluloco-theme` sets `DirectoryIconsColorDefaultDirectory` to a custom color before `yeet-directory-icons.setup()` runs
- **THEN** `yeet-directory-icons` uses the theme-provided color and does not overwrite it

#### Scenario: No theme override uses plugin default
- **WHEN** no theme plugin has set a color for `DirectoryIconsColorRs`
- **THEN** `yeet-directory-icons` sets `DirectoryIconsColorRs` to its built-in Nerd Font default color during `setup()`

### Requirement: Plugin fallback safety
If the plugin's hook handler fails for any reason, the core SHALL preserve the bufferline in its pre-hook state (empty icon column, default text content) so buffer rendering continues without error.

#### Scenario: Plugin hook failure degrades gracefully
- **WHEN** a hook call to the plugin raises an error
- **THEN** the bufferline retains its pre-hook state with no icon and original content

### Requirement: Plugin provides its own help documentation
The `yeet-directory-icons` plugin SHALL include a `docs/help/directory-icons.md` file in its plugin directory that documents all `DirectoryIconsColor*` tokens, their naming convention, default values, usage examples, and configuration guidance. This documentation SHALL be discoverable via `:help directory-icons` when the plugin is loaded.

#### Scenario: Plugin help page accessible via :help
- **WHEN** `yeet-directory-icons` is loaded and the user runs `:help directory-icons`
- **THEN** the plugin's help page is displayed with full token reference and usage examples
