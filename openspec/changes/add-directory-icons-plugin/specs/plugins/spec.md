## ADDED Requirements

### Requirement: Directory icons plugin is vendored as a repository submodule
The repository SHALL include the directory-icons plugin as a git submodule at `plugins/directory-icons` pointing to `git@github.com:aserowy/yeet-directory-icons.git`.

#### Scenario: Submodule path is present
- **WHEN** the repository is checked out with submodules initialized
- **THEN** `plugins/directory-icons` exists as a git submodule entry sourced from the declared repository URL

### Requirement: Plugin identity remains yeet-directory-icons
The vendored plugin SHALL retain the logical/plugin name `yeet-directory-icons` even though the submodule folder name is `plugins/directory-icons`.

#### Scenario: Runtime references yeet-directory-icons identity
- **WHEN** plugin loading registers the vendored directory icon plugin
- **THEN** runtime/plugin identity is `yeet-directory-icons` and submodule path remains `plugins/directory-icons`

### Requirement: Vendored directory-icons plugin is loaded for directory rendering
At startup, plugin loading SHALL make the vendored directory-icons plugin available to directory buffer rendering so icon descriptors can be requested for directory entries.

#### Scenario: Directory rendering can resolve icons through vendored plugin
- **WHEN** yeet starts and opens a directory buffer
- **THEN** icon lookup requests are served by the vendored directory-icons plugin

#### Scenario: Missing vendored plugin reports startup error
- **WHEN** `plugins/directory-icons` is not present on disk at startup
- **THEN** the system reports a plugin loading error indicating the missing vendored plugin path
