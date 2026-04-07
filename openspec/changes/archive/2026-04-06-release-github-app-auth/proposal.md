## Why

The automated release workflow cannot push commits directly to `main` because branch protection rules require pull requests. A GitHub App token is needed to bypass this restriction while keeping the workflow fully automated.

## What Changes

- Add the `actions/create-github-app-token` action to the automated release workflow to generate a token from a GitHub App
- Use the app token for checkout and push operations instead of the default `GITHUB_TOKEN`
- Requires two repository secrets: `APP_ID` and `APP_PRIVATE_KEY` from a GitHub App with contents write permission
- Update release documentation to mention the GitHub App setup requirement

## Capabilities

### New Capabilities

### Modified Capabilities

- `devops`: The release workflow authentication changes from `GITHUB_TOKEN` to a GitHub App token to bypass branch protection

## Impact

- `.github/workflows/automated-release.yml`: Token generation step added, checkout and push steps updated to use app token
- Repository settings: Two new secrets (`APP_ID`, `APP_PRIVATE_KEY`) must be configured
- GitHub App: Must be created with `contents: write` permission and installed on the repository
- The GitHub App must be added to the branch protection ruleset as a bypass actor
