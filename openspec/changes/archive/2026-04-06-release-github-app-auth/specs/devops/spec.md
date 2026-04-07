## MODIFIED Requirements

### Requirement: Version commit is created on main

The workflow SHALL generate a GitHub App installation token using the `actions/create-github-app-token` action with `APP_ID` and `APP_PRIVATE_KEY` secrets. The workflow SHALL use this token for repository checkout and push operations to bypass branch protection rules. The workflow SHALL commit the `Cargo.toml` version change to the `main` branch and tag that commit with the computed version tag.

#### Scenario: Commit and tag on main

- **WHEN** the workflow completes version computation and Cargo.toml update
- **THEN** a new commit SHALL exist on `main` with the updated `Cargo.toml`
- **AND** the computed version tag SHALL point to that commit

#### Scenario: Workflow authenticates with GitHub App token

- **WHEN** the workflow starts
- **THEN** it SHALL generate an installation token using `actions/create-github-app-token` with `APP_ID` and `APP_PRIVATE_KEY` repository secrets
- **AND** use that token for checkout and push operations

#### Scenario: Push bypasses branch protection

- **WHEN** the workflow pushes the version commit and tag to `main`
- **THEN** the push SHALL succeed despite branch protection rules requiring pull requests
