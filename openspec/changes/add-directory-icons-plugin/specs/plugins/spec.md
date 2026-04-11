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

### Requirement: Mutation hook fires for all buffer types with buffer-type metadata
The core SHALL invoke the `on_bufferline_mutate` hook for all buffer types when bufferlines are created or updated. Each hook invocation SHALL provide the buffer type (e.g., `directory`, `content`, `help`, `quickfix`, `tasks`) as metadata alongside the full bufferline context. The plugin decides which buffer types to process.

#### Scenario: Hook fires for directory buffer entries
- **WHEN** the core handles `EnumerationChanged`, `EnumerationFinished`, or `PathsAdded` and processes a bufferline
- **THEN** the hook fires with buffer type `directory` and the parent directory path

#### Scenario: Hook fires for content buffer entries
- **WHEN** the core populates a content buffer (file preview)
- **THEN** the hook fires for each bufferline with buffer type `content` and the file path

#### Scenario: Hook fires for help buffer entries
- **WHEN** the core populates a help buffer
- **THEN** the hook fires for each bufferline with buffer type `help`

#### Scenario: Hook fires for quickfix buffer entries
- **WHEN** the core populates a quickfix buffer
- **THEN** the hook fires for each bufferline with buffer type `quickfix`

#### Scenario: Hook fires for tasks buffer entries
- **WHEN** the core populates a tasks buffer
- **THEN** the hook fires for each bufferline with buffer type `tasks`

#### Scenario: Plugin directly mutates bufferline via hook context
- **WHEN** the plugin receives a hook call with bufferline and buffer-type metadata
- **THEN** the plugin can mutate `prefix`, `content`, `search_char_position`, `signs`, and `icon` fields in-place

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
