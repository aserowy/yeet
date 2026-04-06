# Releasing

Releases are automated via a GitHub Actions workflow. Pushing a `release` tag
triggers the pipeline which computes a calendar-based version tag, updates the
version in `Cargo.toml`, commits to `main`, and creates a GitHub release.

## Creating a release

```sh
git tag release
git push origin release
```

The workflow will:

1. Compute a version tag in the format `vYYYY.M.D` (e.g., `v2026.4.6`)
2. If the tag already exists, increment the day component until an unused tag is
   found (e.g., `v2026.4.7`)
3. Update the workspace version in `Cargo.toml`
4. Commit the change to `main` and push the version tag
5. Delete the `release` tag so it can be reused

The version tag then triggers the existing release and FlakeHub publish
workflows which build platform binaries and create a draft GitHub release.
