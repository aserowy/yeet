## Why

Releases are currently triggered by manually pushing a version tag. This requires remembering the date-based versioning scheme and manually updating `Cargo.toml`. Automating this process reduces human error and streamlines the release workflow to a single `release` tag push.

## What Changes

- Add a new GitHub Actions workflow that triggers on push of a `release` tag
- The workflow computes a calendar-based version tag (`vYYYY.M.D`, no leading zeros) and increments the day component if the tag already exists
- Updates the workspace version in `Cargo.toml` to match the computed version
- Commits the version change to `main`, tags the commit with the computed version, and pushes both
- The existing `release.yml` and `flakehub-publish-tagged.yml` workflows then trigger on the new version tag as before
- **BREAKING**: The release workflow no longer triggers on manually pushed version tags directly — instead, push the `release` tag to initiate

## Capabilities

### New Capabilities

- `automated-release-pipeline`: Workflow that computes a date-based version tag, updates `Cargo.toml`, commits to `main`, and creates the version tag — triggered by pushing a `release` tag

### Modified Capabilities

## Impact

- `.github/workflows/`: New workflow file for the automated release pipeline
- `Cargo.toml`: Workspace version will be updated automatically on each release
- Existing `release.yml` and `flakehub-publish-tagged.yml` remain unchanged — they still trigger on version tags
- Contributors must push the `release` tag instead of a version tag to create a release
