## Context

The existing `dependency-update-compatibility` capability requires accepted Renovate updates to preserve Rust workspace buildability, Nix evaluation/build checks, regression coverage, and the supported application startup path. The current Renovate upgrade has broken one or more of those guarantees and needs compatibility work across the affected dependency integration points.

The repository uses Rust, Lua, Nix, and Markdown. The implementation should be driven by the concrete failures exposed by the supported build tooling and existing project checks, not by broad refactors.

## Goals / Non-Goals

**Goals:**
- Reproduce the current post-Renovate failures with automated commands before applying fixes.
- Restore successful Rust workspace builds and relevant Nix evaluation/build checks.
- Add or identify regression coverage that would fail before the compatibility fix and pass afterward.
- Verify the supported startup or smoke-run path gets far enough to prove dependency initialization succeeds.
- Preserve existing user-facing behavior unless a separate spec change intentionally documents otherwise.

**Non-Goals:**
- Introduce unrelated feature changes or broad architectural refactors.
- Roll back dependency upgrades unless a dependency is demonstrably unusable and no compatible fix exists.
- Change public behavior, commands, UI semantics, or documentation outside what is required for compatibility.

## Decisions

- **Start with reproducible checks before fixes.** Run the supported Rust, Nix, test, and smoke-run commands first and record the failing command(s). Alternative considered: inspect dependency diffs and patch likely call sites directly. Rejected because the spec requires a failing test or reproducible automated check before the compatibility fix.
- **Prefer minimal compatibility adaptations at dependency boundaries.** Update call sites, feature flags, configuration, or wrapper code closest to the changed dependency APIs. Alternative considered: larger internal rewrites. Rejected to minimize behavior risk after a dependency-only upgrade.
- **Keep compatibility verification implementation-focused.** Use existing test suites and add targeted regression tests only where current coverage does not reproduce the failure. Alternative considered: add broad end-to-end coverage for every dependency update. Rejected as excessive for a focused Renovate compatibility fix.

## Risks / Trade-offs

- **Failure source is ambiguous across multiple upgraded dependencies** → Mitigate by isolating failures through build output, lock/config diffs, and targeted checks before changing code.
- **Nix and non-Nix environments can expose different dependency resolution failures** → Mitigate by running the repository-supported Nix evaluation/build check in addition to Rust tooling where available.
- **Smoke-run commands may require environment assumptions** → Mitigate by using the documented supported development/application entry point and recording any unavailable external prerequisites.
- **Minimal fixes may miss related incompatibilities** → Mitigate by running relevant existing tests after the targeted regression passes.
