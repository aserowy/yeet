## MODIFIED Requirements

### Requirement: Workspace version in Cargo.toml is updated

The workflow SHALL update the `version` field under `[workspace.package]` and the `version` fields in workspace dependency entries for internal crates (`yeet-*`) in the root `Cargo.toml` to match the computed version without the `v` prefix. The workflow SHALL NOT modify version strings of external dependencies.

#### Scenario: Version updated in Cargo.toml

- **WHEN** the computed version tag is `v2026.4.2`
- **THEN** the `version` field in `Cargo.toml` under `[workspace.package]` SHALL be set to `"2026.4.2"`

#### Scenario: Internal crate versions updated

- **WHEN** the computed version tag is `v2026.4.2`
- **THEN** the `version` field in `yeet-buffer`, `yeet-frontend`, `yeet-keymap`, and `yeet-lua` entries under `[workspace.dependencies]` SHALL be set to `"2026.4.2"`

#### Scenario: External dependency versions are not modified

- **WHEN** the workflow updates versions in `Cargo.toml`
- **THEN** external dependency version strings (e.g., `arboard = { version = "3.6.1", ... }`) SHALL remain unchanged
