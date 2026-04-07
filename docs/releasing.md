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

1. Compute a version tag in the format `vYYYY.M.PATCH` (e.g., `v2026.4.0`)
   where PATCH is a sequential number starting at 0 for each year-month
   combination
2. Update the workspace version in `Cargo.toml`
3. Commit the change to `main` and push the version tag
4. Delete the `release` tag so it can be reused

The version tag then triggers the existing release and FlakeHub publish
workflows which build platform binaries and create a draft GitHub release.

## GitHub App setup

The workflow uses a GitHub App token to push directly to `main` despite branch
protection rules. This requires a one-time setup:

1. Create a GitHub App with `Contents: Read & write` permission
2. Install the app on the repository
3. Add the app as a bypass actor in the branch protection ruleset for `main`
4. Add the following repository secrets:
   - `APP_ID`: The GitHub App's ID
   - `APP_PRIVATE_KEY`: The GitHub App's private key
