## Context

The automated release workflow pushes a commit and tag directly to `main`, but branch protection rules require pull requests for all pushes. The default `GITHUB_TOKEN` respects these rules and cannot bypass them. A GitHub App can be granted bypass permissions in the repository's branch protection ruleset.

## Goals / Non-Goals

**Goals:**

- Enable the automated release workflow to push to `main` despite branch protection
- Use a GitHub App token for authentication (not tied to a personal account)
- Minimal changes to the existing workflow — only swap the token source

**Non-Goals:**

- Changing the release logic (version computation, Cargo.toml update, tagging)
- Modifying branch protection rules beyond adding the app as a bypass actor
- Automating the GitHub App creation or installation (manual one-time setup)

## Decisions

### 1. Use `actions/create-github-app-token` action

The workflow uses the official `actions/create-github-app-token` action to generate an installation token from the app's credentials. This is the standard approach recommended by GitHub for workflows that need elevated permissions.

**Alternative considered**: Manually generating tokens via API calls in a script step — rejected as unnecessarily complex and harder to maintain.

### 2. Store credentials as repository secrets

The GitHub App's ID is stored as `APP_ID` and the private key as `APP_PRIVATE_KEY` in repository secrets. These are referenced by the token generation step.

**Alternative considered**: Using environment-level secrets — rejected as there's only one environment (production releases).

### 3. Token used for checkout and push only

The generated app token replaces `GITHUB_TOKEN` only in the checkout step (which sets up git credentials) and is used implicitly for push operations. All other steps (version computation, sed, commit) operate locally and don't need the token.

## Risks / Trade-offs

- **GitHub App must be manually created and installed** -> One-time setup documented in release docs. If the app is removed or keys rotated, the workflow fails with a clear auth error.
- **App must be added as bypass actor in branch protection** -> Requires repo admin access. If forgotten, the workflow still fails at push time with a protection error.
- **Secret rotation** -> If the app's private key is rotated, `APP_PRIVATE_KEY` must be updated in repository secrets. The workflow fails immediately with an auth error if stale.

## Migration Plan

1. Create a GitHub App with `contents: write` permission
2. Install the app on the repository
3. Add the app as a bypass actor in the branch protection ruleset for `main`
4. Add `APP_ID` and `APP_PRIVATE_KEY` as repository secrets
5. Deploy the updated workflow
