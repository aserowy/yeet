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
- **THEN** mutation hook calls during `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling are served by `yeet-directory-icons`

#### Scenario: Plugin load sets icon-column width
- **WHEN** `yeet-directory-icons` executes its `on_window_create` hook
- **THEN** shared `@yeet-buffer` icon-column width is configured to `1`

#### Scenario: Plugin unavailable keeps zero-width icon column
- **WHEN** `yeet-directory-icons` is unavailable or not configured
- **THEN** shared `@yeet-buffer` icon-column width remains at the default `0` and no per-bufferline hooks are invoked

#### Scenario: Plugin configuration/load failure is reported
- **WHEN** `yeet-directory-icons` is configured but fails to load
- **THEN** the system reports a plugin loading diagnostic and continues with icon-column width `0`

### Requirement: New mutation hooks in EnumerationChanged/EnumerationFinished/PathsAdded message handling
The core SHALL introduce new hooks in the existing `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling that pass the complete bufferline and the given window with all metadata to the plugin on each hook call. The plugin directly mutates the bufferline in-place (sets icon, colors text). These hooks fire at the same point where directory content is set or updated.

#### Scenario: Mutation hook fires during EnumerationChanged processing
- **WHEN** the core handles an `EnumerationChanged` message and processes a bufferline
- **THEN** a hook is invoked providing the complete bufferline and the given window with all information; the plugin directly mutates the bufferline

#### Scenario: Mutation hook fires during EnumerationFinished processing
- **WHEN** the core handles an `EnumerationFinished` message and processes a bufferline
- **THEN** a hook is invoked providing the complete bufferline and the given window with all information; the plugin directly mutates the bufferline

#### Scenario: Mutation hook fires during PathsAdded processing
- **WHEN** the core handles a `PathsAdded` message and creates a new bufferline for an added path
- **THEN** a hook is invoked providing the complete bufferline and the given window with all information; the plugin directly mutates the bufferline

#### Scenario: Plugin directly mutates bufferline via hook context
- **WHEN** the plugin receives a hook call with bufferline and window context
- **THEN** the plugin adds/replaces the icon in the icon column and colors the bufferline text in-place

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
