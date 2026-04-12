## ADDED Requirements

### Requirement: Plugin identity remains yeet-directory-icons
The integrated plugin SHALL use logical/plugin name `yeet-directory-icons` through the existing plugin configuration/loading flow.

#### Scenario: Runtime references yeet-directory-icons identity
- **WHEN** plugin loading registers the directory icon plugin from user configuration
- **THEN** runtime/plugin identity is `yeet-directory-icons`

### Requirement: Existing plugin loading is used for directory rendering
At startup, existing plugin loading SHALL make `yeet-directory-icons` available to directory buffer rendering so the plugin can mutate bufferlines via hooks.

#### Scenario: Directory rendering invokes plugin mutation hooks through configured plugin
- **WHEN** yeet starts and opens a directory buffer with `yeet-directory-icons` configured and available
- **THEN** mutation hook calls fire for each bufferline and are served by `yeet-directory-icons`

#### Scenario: Plugin load sets icon-column width
- **WHEN** `yeet-directory-icons` executes its `on_window_create` hook
- **THEN** shared `@yeet-buffer` icon-column width is configured to `1`

#### Scenario: Plugin unavailable keeps zero-width icon column
- **WHEN** `yeet-directory-icons` is unavailable or not configured
- **THEN** shared `@yeet-buffer` icon-column width remains at the default `0` and no per-bufferline hooks are invoked

#### Scenario: Plugin configuration/load failure is reported
- **WHEN** `yeet-directory-icons` is configured but fails to load
- **THEN** the system reports a plugin loading diagnostic and continues with icon-column width `0`

### Requirement: Mutation hook fires for all buffer types with buffer metadata object
The core SHALL invoke the `on_bufferline_mutate` hook for all buffer types when bufferlines are created or updated. Each hook invocation SHALL provide buffer metadata as a read-only `buffer` object (`ctx.buffer`) containing `type` (e.g., `"directory"`, `"content"`, `"help"`, `"quickfix"`, `"tasks"`) and optionally `path` (parent dir for directory, file path for content; absent/nil for help, quickfix, tasks). The plugin decides which buffer types to process by checking `ctx.buffer.type`. The `buffer_type` parameter in the Rust API SHALL use a `BufferType` enum instead of `&str`.

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
- **THEN** the plugin can mutate `prefix`, `content`, `search_char_position`, `signs`, and `icon` fields in-place on the context table; the `buffer` metadata object is read-only

### Requirement: Deferred PathsAdded hooks fire on flush
When `PathsAdded` events are deferred during Insert mode, the per-bufferline mutation hooks SHALL also be deferred. Hooks fire when deferred events are flushed (after leaving Insert mode).

#### Scenario: Deferred PathsAdded hooks fire after leaving Insert mode
- **WHEN** `PathsAdded` events are queued during Insert mode and the user leaves Insert mode
- **THEN** the deferred events are flushed and mutation hooks fire for each new bufferline at flush time

### Requirement: Plugin-manager workflows are unchanged
The system SHALL NOT require changes to plugin-manager commands/workflows (install/update/sync/lock) for this feature.

#### Scenario: Feature uses normal plugin configuration path
- **WHEN** a user installs/configures `yeet-directory-icons` through their normal setup
- **THEN** directory icon integration works without introducing new plugin-manager behavior

### Requirement: Plugins can provide help pages
Plugins SHALL be able to provide help documentation by placing markdown files in a `docs/help/` directory within their plugin directory. The `:help` command SHALL discover these files at runtime and include them as searchable help pages.

#### Scenario: Plugin help page is discoverable
- **WHEN** a plugin has a `docs/help/directory-icons.md` file
- **THEN** `:help directory-icons` shows the plugin's help content

#### Scenario: Core help takes priority
- **WHEN** a topic matches both a core help page and a plugin help page
- **THEN** the core help page is shown

#### Scenario: Plugin not loaded means no help
- **WHEN** a plugin is not loaded/configured
- **THEN** its help pages are not available in `:help`

### Requirement: Plugin-specific documentation lives in plugin repos
Plugin-specific documentation (token references, usage guides, configuration) SHALL be maintained in each plugin's own `docs/help/` directory, not in core `docs/help/` files. Core documentation SHALL only document core concepts and SHALL NOT reference optional plugin-specific tokens or behavior.

#### Scenario: Plugin token docs in plugin repo
- **WHEN** a user wants to learn about `DirectoryIconsColor*` tokens
- **THEN** they find the documentation in the `yeet-directory-icons` plugin's help page, not in core `docs/help/theme.md`
