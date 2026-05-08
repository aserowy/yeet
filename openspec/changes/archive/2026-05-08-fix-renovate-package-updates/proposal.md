## Why

Renovate has updated project dependencies and the application is no longer buildable or runnable. We need a test-first compatibility repair so dependency update regressions are captured before fixing build, runtime, and integration failures.

## What Changes

- Add regression tests that reproduce the current dependency-update breakage before applying fixes.
- Restore successful workspace build and test execution after updated Rust, Lua, Nix, or related package versions.
- Update code and configuration for changed dependency APIs or behavior without changing user-facing product behavior.
- Ensure the project can run successfully after fixes using the existing development/runtime entry points.

## Capabilities

### New Capabilities
- `dependency-update-compatibility`: Captures expectations for keeping the project buildable, testable, and runnable after automated dependency updates.

### Modified Capabilities

## Impact

- Affected areas may include Cargo workspace dependencies and lockfile, Rust source using updated APIs, Lua integration, Nix flake/dev-shell configuration, and CI/build commands.
- No intentional user-facing API or behavior changes are expected; fixes should preserve existing capabilities while restoring compatibility with updated packages.
