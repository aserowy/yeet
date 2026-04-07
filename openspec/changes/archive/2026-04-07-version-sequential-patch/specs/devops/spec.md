## MODIFIED Requirements

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

The workflow SHALL update the `version` field under `[workspace.package]` in the root `Cargo.toml` to match the computed version without the `v` prefix.

#### Scenario: Version updated in Cargo.toml

- **WHEN** the computed version tag is `v2026.4.2`
- **THEN** the `version` field in `Cargo.toml` under `[workspace.package]` SHALL be set to `"2026.4.2"`

## REMOVED Requirements

### Requirement: Tag collision increments day component

**Reason**: Replaced by sequential patch numbering. The day component is no longer used in version tags, so collision by incrementing the day is no longer applicable.
**Migration**: The new "Version tag follows calendar versioning" requirement handles uniqueness via sequential patch numbers instead.
