## ADDED Requirements

### Requirement: Plugin spec includes help page paths
The `PluginSpec` struct SHALL include a `help_pages` field containing a list of resolved help page file paths. Each entry SHALL represent a markdown file found in the plugin's `docs/help/` directory.

#### Scenario: Plugin with help pages
- **WHEN** a plugin has files `docs/help/directory-icons.md` and `docs/help/advanced.md`
- **THEN** the `PluginSpec.help_pages` field SHALL contain paths to both files

#### Scenario: Plugin without help pages
- **WHEN** a plugin has no `docs/help/` directory or the directory is empty
- **THEN** the `PluginSpec.help_pages` field SHALL be an empty list

#### Scenario: Plugin directory does not exist
- **WHEN** a plugin is registered but its directory does not exist on disk
- **THEN** the `PluginSpec.help_pages` field SHALL be an empty list

### Requirement: Help page discovery happens at spec initialization
Plugin help pages SHALL be discovered during spec initialization in `yeet-plugin` (Rust side), not at runtime in the frontend. The discovery SHALL scan each plugin's resolved storage directory for `docs/help/*.md` files and populate the `help_pages` field on the spec.

#### Scenario: Help pages resolved during initialization
- **WHEN** `yeet-plugin` initializes plugin specs from the registered plugin list
- **THEN** each spec's `help_pages` field is populated by scanning the plugin's `docs/help/` directory

#### Scenario: Frontend reads help pages from spec
- **WHEN** the frontend needs to discover plugin help pages for `:help` resolution
- **THEN** it reads the `help_pages` field from `PluginSpec` instead of scanning the filesystem

### Requirement: Plugin-specific documentation lives in plugin repos
Plugin-specific documentation (token references, usage guides, configuration) SHALL be maintained in each plugin's own `docs/help/` directory, not in core `docs/help/` files. Core documentation SHALL only document core concepts and SHALL NOT reference optional plugin-specific tokens or behavior.

#### Scenario: Plugin token docs in plugin repo
- **WHEN** a user wants to learn about `DirectoryIconsColor*` tokens
- **THEN** they find the documentation in the `yeet-directory-icons` plugin's help page, not in core `docs/help/theme.md`

## MODIFIED Requirements

### Requirement: Mutation hook fires for all buffer types with buffer metadata object
The core SHALL invoke the `on_bufferline_mutate` hook for all buffer types when bufferlines are created or updated. Each hook invocation SHALL provide buffer metadata as a read-only `buffer` object (`ctx.buffer`) containing `type` (e.g., `"directory"`, `"content"`, `"help"`, `"quickfix"`, `"tasks"`) and optionally `path` (parent dir for directory, file path for content; absent/nil for help, quickfix, tasks). The plugin decides which buffer types to process by checking `ctx.buffer.type`. The `buffer_type` parameter in the Rust API SHALL use a `BufferType` enum instead of `&str`. The mutable fields in the hook context are: `prefix`, `content`, `search_char_position`, and `signs`. The `icon` field is no longer part of the hook context.

#### Scenario: Hook fires for directory buffer entries
- **WHEN** the core handles `EnumerationChanged`, `EnumerationFinished`, or `PathsAdded` and processes a bufferline
- **THEN** the hook fires with `ctx.buffer.type` set to `"directory"` and `ctx.buffer.path` set to the parent directory path

#### Scenario: Hook fires for content buffer entries
- **WHEN** the core populates a content buffer (file preview)
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"content"` and `ctx.buffer.path` set to the file path

#### Scenario: Hook fires for help buffer entries
- **WHEN** the core populates a help buffer
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"help"` and `ctx.buffer.path` absent (nil)

#### Scenario: Hook fires for quickfix buffer entries
- **WHEN** the core populates a quickfix buffer
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"quickfix"` and `ctx.buffer.path` absent (nil)

#### Scenario: Hook fires for tasks buffer entries
- **WHEN** the core populates a tasks buffer
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"tasks"` and `ctx.buffer.path` absent (nil)

#### Scenario: Plugin directly mutates bufferline via hook context
- **WHEN** the plugin receives a hook call with bufferline and `ctx.buffer` metadata object
- **THEN** the plugin can mutate `prefix`, `content`, `search_char_position`, and `signs` fields in-place on the context table; the `buffer` metadata object is read-only

### Requirement: Plugins can provide help pages
Plugins SHALL be able to provide help documentation by placing markdown files in a `docs/help/` directory within their plugin directory. The `:help` command SHALL discover these files via the `help_pages` field on `PluginSpec` (populated at spec initialization) and include them as searchable help pages.

#### Scenario: Plugin help page is discoverable
- **WHEN** a plugin has a `docs/help/directory-icons.md` file
- **THEN** `:help directory-icons` shows the plugin's help content

#### Scenario: Core help takes priority
- **WHEN** a topic matches both a core help page and a plugin help page
- **THEN** the core help page is shown

#### Scenario: Plugin not loaded means no help
- **WHEN** a plugin is not loaded/configured
- **THEN** its help pages are not available in `:help`
