## Why

`nix build .` fails because two tests (`split_no_args_from_tasks_falls_back_to_home` and `vsplit_no_args_from_tasks_falls_back_to_home`) assume `dirs::home_dir()` returns an existing directory. In the Nix sandbox, the home directory resolves to `/homeless-shelter`, which does not exist on disk, causing the tests to fail when the code checks `target_path.exists()`.

## What Changes

- Fix the two failing tests in `yeet-frontend/src/update/command/mod.rs` to account for environments where the home directory does not exist on disk.
- The test conditions currently check `dirs::home_dir().is_some()` but should check `dirs::home_dir().filter(|p| p.exists()).is_some()` to match the runtime behavior of the code under test.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `devops`: Test robustness requirement — tests must not assume the home directory exists on disk.

## Impact

- Affected code: `yeet-frontend/src/update/command/mod.rs` (test module)
- Affected system: Nix build pipeline — this is the primary build method for the package
- No API or user-facing behavior changes
