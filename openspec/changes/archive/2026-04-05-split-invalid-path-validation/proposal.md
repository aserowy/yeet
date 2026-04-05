## Why

When executing `:split asdf` or `:vsplit asdf` where `asdf` is not a valid relative path (i.e., the resolved target directory does not exist on disk), the application creates a split window pointing to a non-existent path. This produces a bugged buffer instead of showing an error message. The `expand_path()` function resolves relative paths by joining them with the current directory but does not validate that the result exists.

## What Changes

- Add path existence validation in the `split` and `vsplit` command handlers after `expand_path()` succeeds, returning an error if the resolved target path does not exist as a directory.

## Capabilities

### New Capabilities

### Modified Capabilities

- `window-management`: Add validation that the split target path exists before creating the split window.

## Impact

- `yeet-frontend/src/update/command/mod.rs`: Add existence check for resolved target path in `split` and `vsplit` handlers.
- No API or dependency changes.
- No breaking changes.
