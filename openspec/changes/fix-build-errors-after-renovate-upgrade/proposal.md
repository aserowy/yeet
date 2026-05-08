## Why

Renovate dependency updates have introduced compatibility failures that prevent the project from satisfying its supported build, test, and run paths. This change is needed now to restore confidence that accepted dependency upgrades keep the repository buildable, testable, and runnable.

## What Changes

- Identify the current build, dependency resolution, test, and smoke-run failures introduced after the Renovate upgrade.
- Add or identify reproducible automated regression coverage before applying compatibility fixes.
- Update affected Rust, Lua, Nix, lock/configuration, or integration code to work with the upgraded dependencies.
- Verify the supported build tooling, relevant tests, and application startup path pass after the fixes.

## Capabilities

### New Capabilities


### Modified Capabilities
- `dependency-update-compatibility`: Adds a concrete compatibility-fix change for the current Renovate upgrade while preserving existing buildability, testability, and runnability requirements.

## Impact

- Affected systems may include Rust workspace code and dependency APIs, Nix build/evaluation metadata, Lua integration points, lockfiles, and regression/smoke checks.
- No intentional user-facing behavior changes are expected; fixes should preserve existing behavior covered by current capabilities.
