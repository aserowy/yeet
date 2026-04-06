## Context

Releases currently require manually choosing a version tag, updating `Cargo.toml`, and pushing the tag. The project uses calendar versioning (`vYYYY.M.D`) with existing tags like `v2026.3.10`, `v2026.2.22`, etc. The existing `release.yml` and `flakehub-publish-tagged.yml` workflows trigger on version tag patterns and handle building/publishing. The workspace version in the root `Cargo.toml` is currently `0.1.0` and is not kept in sync with release tags.

## Goals / Non-Goals

**Goals:**

- Automate version computation using calendar versioning (`vYYYY.M.D`, no leading zeros)
- Handle tag collisions by incrementing the day component (e.g., `v2026.4.6` taken -> `v2026.4.7`)
- Update the workspace version in `Cargo.toml` to match the release tag (without `v` prefix)
- Commit the version update to `main`, tag the commit, and push — triggering existing release workflows
- Trigger the entire flow by pushing a `release` tag

**Non-Goals:**

- Changing the existing `release.yml` or `flakehub-publish-tagged.yml` workflows
- Changing the build matrix or artifact packaging
- Supporting manual version overrides in this workflow

## Decisions

### 1. Single workflow triggered by `release` tag

The new workflow triggers on push of a tag named exactly `release`. This is a simple, memorable trigger that avoids conflicts with version tag patterns. After the workflow runs, it deletes the `release` tag so it can be reused for the next release.

**Alternative considered**: Using `workflow_dispatch` — rejected because pushing a tag is simpler from the CLI and doesn't require GitHub UI or API tokens with different scopes.

### 2. Version computation in shell

The version is computed using `date` command (`date +%-Y.%-m.%-d`) which natively produces no-leading-zero formatting on Linux. Tag existence is checked with `git tag -l`, and on collision the day component is incremented in a loop.

**Alternative considered**: A dedicated versioning tool or action — rejected as overkill for simple date-based computation.

### 3. Direct push to main with version update

The workflow checks out `main`, updates `Cargo.toml` using `sed`, commits, tags, and pushes. This keeps the version in `Cargo.toml` synchronized with the release tag. The workflow uses a GitHub Actions token with write permissions.

**Alternative considered**: Creating the tag without updating `Cargo.toml` — rejected because keeping the version in sync with releases is an explicit requirement.

### 4. Cargo.toml update via sed

The workspace version field in the root `Cargo.toml` is updated using `sed` to replace the version string. Since all workspace members inherit from `workspace.package.version`, only one file needs updating.

## Risks / Trade-offs

- **Race condition on concurrent pushes to main** -> The workflow does a `git pull --rebase` before pushing. If this fails, the workflow fails and the release tag can be re-pushed.
- **Release tag reuse** -> The workflow deletes the `release` tag after use, enabling reuse. If deletion fails, the next push of `release` will require force-deleting the old tag first.
- **Day increment can produce non-real dates** (e.g., `v2026.4.32`) -> This is explicitly acceptable per requirements and matches the existing convention of treating the version as an opaque identifier rather than a strict date.
