## 1. Reproduce and Scope Failures

- [ ] 1.1 Run the supported Rust workspace build/check command and record the current Renovate-related failure output.
- [ ] 1.2 Run the repository-supported Nix evaluation or build check and record any configuration or dependency resolution failures.
- [ ] 1.3 Run the relevant existing test suites and identify whether they already reproduce the compatibility failure.
- [ ] 1.4 Run the supported application startup or smoke-run command far enough to identify dependency initialization failures.

## 2. Regression Coverage

- [ ] 2.1 Add or identify a reproducible automated check that fails before the compatibility fix for each confirmed failure category.
- [ ] 2.2 Keep the regression coverage focused on dependency compatibility and avoid unrelated behavior changes.

## 3. Compatibility Fixes

- [ ] 3.1 Update affected Rust dependency API usage, features, or configuration to compile with the upgraded dependency set.
- [ ] 3.2 Update affected Nix package metadata, inputs, or build configuration to evaluate and build with the upgraded dependency set.
- [ ] 3.3 Update affected Lua integration or runtime initialization code if the smoke-run failure traces to upgraded dependency behavior.
- [ ] 3.4 Remove any temporary diagnostic code and ensure no unrelated user-facing behavior changes were introduced.

## 4. Verification

- [ ] 4.1 Run `cargo fmt` and fix any formatting changes required.
- [ ] 4.2 Run `cargo clippy` and fix all warnings or errors.
- [ ] 4.3 Run `cargo test` and ensure regression coverage plus relevant existing tests pass.
- [ ] 4.4 Run `git add -A && nix build .` and fix any remaining build or dependency resolution errors.
- [ ] 4.5 Re-run the supported application startup or smoke-run command and verify dependency initialization succeeds without crashing.
