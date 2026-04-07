## 1. Update Workflow

- [x] 1.1 Add `actions/create-github-app-token` step to generate an installation token from `APP_ID` and `APP_PRIVATE_KEY` secrets
- [x] 1.2 Update the checkout step to use the generated app token instead of `GITHUB_TOKEN`
- [x] 1.3 Update the push and tag deletion steps to use the app token

## 2. Documentation

- [x] 2.1 Update `docs/releasing.md` to document the GitHub App setup requirements (app creation, installation, secrets, branch protection bypass)
