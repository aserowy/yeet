# Capability: Automated Release Pipeline

## Purpose

Automate the release process by triggering on a `release` tag push, computing a calendar-versioned tag, updating workspace version metadata, and coordinating with existing release workflows.

## Requirements

### Requirement: Release workflow triggers on release tag

The release pipeline SHALL trigger when a tag named exactly `release` is pushed to the repository.

#### Scenario: Push release tag triggers workflow

- **WHEN** a tag named `release` is pushed
- **THEN** the automated release workflow SHALL start

#### Scenario: Other tags do not trigger workflow

- **WHEN** a tag with any other name (e.g., `v2026.4.6`, `test`) is pushed
- **THEN** the automated release workflow SHALL NOT start

### Requirement: Version tag follows calendar versioning

The workflow SHALL compute a version tag in the format `vYYYY.M.PATCH` where `YYYY` is the four-digit year, `M` is the month without leading zeros, and `PATCH` is a sequential number starting at 0 for each year-month combination.

#### Scenario: First release in a new month

- **WHEN** the workflow runs in April 2026 and no tags matching `v2026.4.*` exist
- **THEN** the computed version tag SHALL be `v2026.4.0`

#### Scenario: Subsequent release in the same month

- **WHEN** the workflow runs in April 2026 and tags `v2026.4.0` and `v2026.4.1` exist
- **THEN** the computed version tag SHALL be `v2026.4.2`

#### Scenario: First release in January 2027

- **WHEN** the workflow runs in January 2027 and no tags matching `v2027.1.*` exist
- **THEN** the computed version tag SHALL be `v2027.1.0`

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

### Requirement: Version commit is created on main

The workflow SHALL generate a GitHub App installation token using the `actions/create-github-app-token` action with `APP_ID` and `APP_PRIVATE_KEY` secrets. The workflow SHALL use this token for repository checkout and push operations to bypass branch protection rules. The workflow SHALL commit the `Cargo.toml` version change to the `main` branch and tag that commit with the computed version tag.

#### Scenario: Commit and tag on main

- **WHEN** the workflow completes version computation and Cargo.toml update
- **THEN** a new commit SHALL exist on `main` with the updated `Cargo.toml`
- **AND** the computed version tag SHALL point to that commit

#### Scenario: Workflow authenticates with GitHub App token

- **WHEN** the workflow starts
- **THEN** it SHALL generate an installation token using `actions/create-github-app-token` with `APP_ID` and `APP_PRIVATE_KEY` repository secrets
- **AND** use that token for checkout and push operations

#### Scenario: Push bypasses branch protection

- **WHEN** the workflow pushes the version commit and tag to `main`
- **THEN** the push SHALL succeed despite branch protection rules requiring pull requests

### Requirement: Release tag is deleted after use

The workflow SHALL delete the `release` tag from the remote after processing so it can be reused for subsequent releases.

#### Scenario: Release tag cleaned up

- **WHEN** the workflow finishes creating the version tag
- **THEN** the `release` tag SHALL be deleted from the remote repository

### Requirement: Existing release workflows trigger on version tag

The version tag created by this workflow SHALL match the pattern expected by the existing `release.yml` and `flakehub-publish-tagged.yml` workflows so they trigger automatically.

#### Scenario: Release build triggered

- **WHEN** the version tag `v2026.4.6` is pushed
- **THEN** the existing `release.yml` workflow SHALL trigger (matches pattern `v[0-9]+.[0-9]+.[0-9]+`)
- **AND** the existing `flakehub-publish-tagged.yml` workflow SHALL trigger (matches pattern `v?[0-9]+.[0-9]+.[0-9]+*`)

### Requirement: Workspace-internal crates use workspace dependency inheritance

All inter-package dependencies within the workspace SHALL be declared in `[workspace.dependencies]` in the root `Cargo.toml` with both `path` and `version.workspace = true`. Member crates SHALL reference them using `workspace = true`.

#### Scenario: Internal crate dependency declaration

- **WHEN** `yeet-frontend` depends on `yeet-buffer`
- **THEN** the root `Cargo.toml` SHALL have `yeet-buffer = { path = "yeet-buffer", version.workspace = true }` in `[workspace.dependencies]`
- **AND** `yeet-frontend/Cargo.toml` SHALL have `yeet-buffer.workspace = true` in `[dependencies]`

#### Scenario: Version consistency across workspace

- **WHEN** the workspace version is updated
- **THEN** all inter-package version references SHALL automatically reflect the new version through workspace inheritance
