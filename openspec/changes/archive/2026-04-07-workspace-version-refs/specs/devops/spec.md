## ADDED Requirements

### Requirement: Workspace-internal crates use workspace dependency inheritance

All inter-package dependencies within the workspace SHALL be declared in `[workspace.dependencies]` in the root `Cargo.toml` with both `path` and `version.workspace = true`. Member crates SHALL reference them using `workspace = true`.

#### Scenario: Internal crate dependency declaration

- **WHEN** `yeet-frontend` depends on `yeet-buffer`
- **THEN** the root `Cargo.toml` SHALL have `yeet-buffer = { path = "yeet-buffer", version.workspace = true }` in `[workspace.dependencies]`
- **AND** `yeet-frontend/Cargo.toml` SHALL have `yeet-buffer.workspace = true` in `[dependencies]`

#### Scenario: Version consistency across workspace

- **WHEN** the workspace version is updated
- **THEN** all inter-package version references SHALL automatically reflect the new version through workspace inheritance
