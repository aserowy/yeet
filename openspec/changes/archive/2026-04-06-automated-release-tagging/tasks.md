## 1. Create Automated Release Workflow

- [x] 1.1 Create `.github/workflows/automated-release.yml` triggered on push of the `release` tag
- [x] 1.2 Implement version computation using `date +%-Y.%-m.%-d` to produce `vYYYY.M.D` format
- [x] 1.3 Implement tag collision loop: check if tag exists with `git tag -l`, increment day component if taken
- [x] 1.4 Update `[workspace.package] version` in root `Cargo.toml` using `sed` with the computed version (without `v` prefix)
- [x] 1.5 Commit the `Cargo.toml` change to `main`, create the version tag, and push both to remote
- [x] 1.6 Delete the `release` tag from the remote after the version tag is pushed

## 2. Documentation

- [x] 2.1 Update relevant documentation in `./docs` describing the new release process (push `release` tag to trigger a release)
