## 1. Failure Reproduction and Test Coverage

- [ ] 1.1 Run the existing build/test/run commands to identify the concrete failures caused by Renovate updates.
- [ ] 1.2 Add or identify failing regression tests or automated checks that capture the observed build, Nix, or startup failures before applying fixes.
- [ ] 1.3 Verify the new regression coverage fails for the expected dependency-update issue and record the failure mode in implementation notes or test names.

## 2. Compatibility Fixes

- [ ] 2.1 Update Rust source, Cargo metadata, or lockfile usage for changed dependency APIs/features while preserving existing behavior.
- [ ] 2.2 Update Lua integration or related runtime initialization code if the dependency changes affect startup behavior.
- [ ] 2.3 Update Nix flake/build configuration for changed package inputs or build requirements without downgrading dependencies unless necessary.
- [ ] 2.4 If an updated dependency must be pinned or downgraded, document the rationale in the implementation and keep a regression check covering the incompatibility.

## 3. Verification

- [ ] 3.1 Run `cargo fmt` and fix any formatting differences.
- [ ] 3.2 Run `cargo clippy` and fix all reported warnings or errors.
- [ ] 3.3 Run `cargo test` and ensure the regression coverage and existing relevant tests pass.
- [ ] 3.4 Run the supported application startup or smoke-run command to verify the project is runnable after fixes.
- [ ] 3.5 Run `git add -A && nix build .` and fix any Nix build failures.
- [ ] 3.6 If README.md or files under ./docs changed, run `markdownlint` on README.md and ./docs markdown files and fix all reported issues.
