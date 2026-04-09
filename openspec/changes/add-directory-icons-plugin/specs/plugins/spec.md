## ADDED Requirements

### Requirement: Plugin identity remains yeet-directory-icons
The integrated plugin SHALL use logical/plugin name `yeet-directory-icons` through the existing plugin configuration/loading flow.

#### Scenario: Runtime references yeet-directory-icons identity
- **WHEN** plugin loading registers the directory icon plugin from user configuration
- **THEN** runtime/plugin identity is `yeet-directory-icons`

### Requirement: Existing plugin loading is used for directory rendering
At startup, existing plugin loading SHALL make `yeet-directory-icons` available to directory buffer rendering so icon descriptors can be requested for file and directory entries.

#### Scenario: Directory rendering can resolve icons through configured plugin
- **WHEN** yeet starts and opens a directory buffer with `yeet-directory-icons` configured and available
- **THEN** icon lookup requests are served by `yeet-directory-icons`

#### Scenario: Plugin load sets icon-column width
- **WHEN** `yeet-directory-icons` executes its `on_window_create` hook
- **THEN** shared `@yeet-buffer` icon-column width is configured to `1`

#### Scenario: Plugin unavailable keeps zero-width icon column
- **WHEN** `yeet-directory-icons` is unavailable or not configured
- **THEN** shared `@yeet-buffer` icon-column width remains at the default `0`

#### Scenario: Plugin configuration/load failure is reported
- **WHEN** `yeet-directory-icons` is configured but fails to load
- **THEN** the system reports a plugin loading diagnostic and continues with icon-column width `0`

### Requirement: Plugin-manager workflows are unchanged
The system SHALL NOT require changes to plugin-manager commands/workflows (install/update/sync/lock) for this feature.

#### Scenario: Feature uses normal plugin configuration path
- **WHEN** a user installs/configures `yeet-directory-icons` through their normal setup
- **THEN** directory icon integration works without introducing new plugin-manager behavior
