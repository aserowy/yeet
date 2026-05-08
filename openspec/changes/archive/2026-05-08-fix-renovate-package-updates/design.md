## Context

Automated dependency updates have left the Rust/Lua/Nix project in a non-runnable state. The fix must begin by encoding the observed failures as tests or verification checks, then make the smallest compatibility changes needed for the updated dependency set while preserving existing behavior.

The repository already uses OpenSpec capability specs, Rust workspace tooling, Nix configuration, and markdown documentation. Because this change is a repair of dependency compatibility rather than a new user-facing feature, documentation changes are only needed if any user-visible command, behavior, or setup instruction changes during implementation.

## Goals / Non-Goals

**Goals:**
- Capture current build/runtime failures with tests or reproducible verification steps before fixing them.
- Restore workspace compilation and the relevant test suite under the updated package versions.
- Restore the project's normal runnable entry point after dependency updates.
- Keep implementation changes narrowly focused on API/configuration compatibility.

**Non-Goals:**
- Introduce new user-facing features or intentional behavior changes.
- Downgrade Renovate updates unless an updated package is incompatible and no practical code/configuration fix exists.
- Rewrite architecture unrelated to the dependency-update failures.

## Decisions

- Use a test-first repair loop. Add or update failing tests/checks that demonstrate the breakage before code fixes, because the user's request explicitly requires tests in advance and this prevents silent regressions after compatibility patches.
- Prefer adapting code/configuration to new dependency APIs over pinning old versions. This keeps Renovate updates valuable and avoids accumulating stale dependencies. Pinning or downgrading is reserved for cases where upstream changes are incompatible with project constraints.
- Treat buildability, testability, and runnability as separate gates. `cargo test` may pass while Nix or runtime startup remains broken, so implementation should verify each gate that the repository supports.
- Preserve existing capability requirements. Any fixes that alter user-visible behavior must either be avoided or documented and reflected in the appropriate existing capability specs before implementation continues.

## Risks / Trade-offs

- Updated dependency APIs may require broad call-site changes → Mitigation: make the smallest mechanical adaptation and rely on existing tests plus new regression coverage.
- Runtime failure may require environment-specific resources in Nix or terminal contexts → Mitigation: prefer deterministic smoke tests or checks that validate startup paths without depending on a full interactive terminal session when possible.
- Some breakage may be caused by an upstream dependency bug → Mitigation: document the finding, pin only the affected dependency with rationale, and add a regression check to make future unpinning safe.
- Test-first work may require creating checks around currently uncompilable code → Mitigation: add focused compile/runtime regression tests or command checks first, then repair the build until they pass.
