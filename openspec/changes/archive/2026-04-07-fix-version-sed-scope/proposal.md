## Why

The release workflow's `sed` command matches all `version = "X.Y.Z"` patterns in `Cargo.toml`, which incorrectly replaces external dependency version strings (e.g., `arboard = { version = "3.6.1", ... }`) alongside the intended workspace and internal crate versions.

## What Changes

- Replace the blanket `sed` replacement with a targeted approach that only updates:
  1. The `version` line under `[workspace.package]`
  2. The `version` field in workspace dependency entries for internal crates (`yeet-buffer`, `yeet-frontend`, `yeet-keymap`, `yeet-lua`)

## Capabilities

### New Capabilities

### Modified Capabilities

- `devops`: The version update step must be scoped to only workspace and internal crate versions

## Impact

- `.github/workflows/automated-release.yml`: The `sed` command in the "update version in Cargo.toml" step
