## Why

The inter-package dependencies (e.g., `yeet-frontend` depending on `yeet-buffer`) only specify `path` but no `version`. This means that when the release workflow updates the workspace version in the root `Cargo.toml`, the inter-package references don't carry version metadata. Adding version constraints ensures consistency across the workspace and is required for publishing to crates.io.

## What Changes

- Add `version.workspace = true` to all inter-package dependency declarations in workspace member `Cargo.toml` files
- Register workspace-internal crates in `[workspace.dependencies]` in the root `Cargo.toml` so members can reference them with `workspace = true`
- Update `Cargo.lock` to reflect the changes
- Update the release workflow to also update inter-package version references when bumping the version

## Capabilities

### New Capabilities

### Modified Capabilities

- `devops`: The release workflow must update inter-package version references alongside the workspace version

## Impact

- `Cargo.toml` (root): Add workspace-internal crates to `[workspace.dependencies]`
- `yeet/Cargo.toml`: Add version to yeet-frontend, yeet-lua dependencies
- `yeet-frontend/Cargo.toml`: Add version to yeet-buffer, yeet-keymap, yeet-lua dependencies
- `yeet-keymap/Cargo.toml`: Add version to yeet-buffer dependency
- `yeet-lua/Cargo.toml`: Add version to yeet-buffer dependency
- `Cargo.lock`: Refreshed
- `.github/workflows/automated-release.yml`: sed command updated to handle all version references
