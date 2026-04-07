## Context

The `nix build .` command is the primary build method for the yeet package. It currently fails because two tests in `yeet-frontend/src/update/command/mod.rs` assume that `dirs::home_dir()` returns a path that exists on disk. In the Nix build sandbox, the HOME environment variable points to `/homeless-shelter`, which does not exist as a real directory. The code under test correctly checks `target_path.exists()` and returns an error when the path doesn't exist, but the test conditions only check `dirs::home_dir().is_some()`, creating a mismatch.

## Goals / Non-Goals

**Goals:**

- Make both failing tests pass in the Nix build sandbox
- Preserve existing test coverage for the split/vsplit home directory fallback behavior
- Keep the fix minimal and localized to the test conditions

**Non-Goals:**

- Changing the runtime behavior of split/vsplit commands
- Refactoring the `expand_path_without_source` function
- Adding new test infrastructure or mock frameworks

## Decisions

**Decision: Fix test conditions to match runtime behavior**

Change the test branch condition from `dirs::home_dir().is_some()` to `dirs::home_dir().filter(|p| p.exists()).is_some()` in both tests.

This mirrors the actual code path: even when `dirs::home_dir()` returns `Some(path)`, the code calls `target_path.exists()` and produces an error if the path doesn't exist. The test conditions must account for this same check.

Alternative considered: Using `tempdir` as a mock home directory. Rejected because it would require either environment variable manipulation (fragile in parallel test execution) or refactoring the production code to accept an injected home directory (over-engineering for a test fix).

Alternative considered: Skipping the tests in Nix with `#[cfg_attr(...)]`. Rejected because the tests should run in all environments — they just need correct expectations.

## Risks / Trade-offs

[Risk: Test no longer exercises the success path in Nix sandbox] → Acceptable. In Nix, the test will exercise the error path instead. The success path is still exercised when running `cargo test` in a normal environment with a real home directory. Both branches are tested across environments.
