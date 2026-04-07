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

The workflow SHALL compute a version tag in the format `vYYYY.M.D` where `YYYY` is the four-digit year, `M` is the month without leading zeros, and `D` is the day without leading zeros.

#### Scenario: Version tag for a release on April 6, 2026

- **WHEN** the workflow runs on 2026-04-06
- **THEN** the computed version tag SHALL be `v2026.4.6`

#### Scenario: Version tag for a release on January 1, 2027

- **WHEN** the workflow runs on 2027-01-01
- **THEN** the computed version tag SHALL be `v2027.1.1`

### Requirement: Tag collision increments day component

When the computed version tag already exists, the workflow SHALL increment the day component by one and check again, repeating until an unused tag is found.

#### Scenario: Tag already exists for today

- **WHEN** the workflow runs on 2026-04-06 and tag `v2026.4.6` already exists
- **THEN** the workflow SHALL use `v2026.4.7` as the version tag

#### Scenario: Multiple tags exist for consecutive days

- **WHEN** the workflow runs on 2026-4-6 and both `v2026.4.6` and `v2026.4.7` already exist
- **THEN** the workflow SHALL use `v2026.4.8` as the version tag

#### Scenario: Increment beyond real calendar days

- **WHEN** the workflow runs on 2026-4-30 and tag `v2026.4.30` already exists
- **THEN** the workflow SHALL use `v2026.4.31` even though April has only 30 days

### Requirement: Workspace version in Cargo.toml is updated

The workflow SHALL update the `version` field under `[workspace.package]` in the root `Cargo.toml` to match the computed version without the `v` prefix.

#### Scenario: Version updated in Cargo.toml

- **WHEN** the computed version tag is `v2026.4.6`
- **THEN** the `version` field in `Cargo.toml` under `[workspace.package]` SHALL be set to `"2026.4.6"`

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
