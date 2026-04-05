## Context

The `split` and `vsplit` command handlers in `command/mod.rs` call `file::expand_path()` to resolve the user-provided path argument. `expand_path()` handles mark expansion and relative path resolution but does not validate that the resulting path exists on disk. The resolved path is passed directly to `split::horizontal()` / `split::vertical()`, which creates a new window pane and emits `NavigateToPath`. When the path doesn't exist, this creates a bugged buffer.

## Goals / Non-Goals

**Goals:**
- Validate that the resolved target path exists as a directory before creating a split.
- Return a clear error message when the target path does not exist.

**Non-Goals:**
- Changing `expand_path()` itself — it's used by other commands (`cp`, `mv`) where the target may legitimately not exist yet.
- Validating paths in the `split::horizontal/vertical` functions — validation belongs in the command handler where user intent is clear.

## Decisions

**Add existence check in the split/vsplit handlers, not in `expand_path()` or `split::create_split()`.**

After `expand_path()` succeeds, check `target_path.exists()` before calling `split::horizontal/vertical`. If the path doesn't exist, return an error via `add_change_mode()`.

Alternative considered: Adding validation inside `expand_path()`. Rejected because `expand_path()` is also used by `cp` and `mv` where the target directory may not exist yet (they validate differently via `expand_and_validate_path()`).

## Risks / Trade-offs

- [Low risk] Localized change to 2 match arms in `execute()`. No impact on other commands.
