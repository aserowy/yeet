## Context

The workspace has 5 member crates with inter-package dependencies that use `path` only. The workspace version is managed in the root `Cargo.toml` under `[workspace.package]` and inherited by all members via `version.workspace = true`. However, the inter-package dependency declarations don't reference the workspace version.

## Goals / Non-Goals

**Goals:**

- Add workspace-internal crates to `[workspace.dependencies]` with path and version
- Convert all inter-package dependency declarations to use `workspace = true`
- Update `Cargo.lock` after changes
- Ensure the release workflow's `sed` command still correctly updates the version

**Non-Goals:**

- Updating external dependency versions
- Publishing to crates.io (just preparing the metadata)

## Decisions

### 1. Use workspace dependency inheritance for internal crates

Register each workspace member crate in `[workspace.dependencies]` in the root `Cargo.toml` with both `path` and `version.workspace = true`. Then each member references them as `crate-name.workspace = true`. This centralizes version management and ensures consistency.

**Alternative considered**: Adding `version = "X.Y.Z"` inline in each member's `Cargo.toml` — rejected because it duplicates the version string and requires updating multiple files on each release.

### 2. Release workflow sed scope is sufficient

The release workflow's `sed` command updates `version = "..."` in the root `Cargo.toml`. Since workspace dependency entries for internal crates won't have a separate `version = "..."` field (they use `version.workspace = true`), the single sed replacement on the `[workspace.package]` version line is sufficient. No workflow changes needed.

## Risks / Trade-offs

- **Workspace dependency with path + version** -> Cargo uses `path` for local development and `version` for publishing. This is standard Cargo behavior and well-supported.
