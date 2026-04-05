## Context

When a command is executed via the commandline (`:split asdf`, `:vsplit badpath`), `command::execute()` processes the command and returns a `Vec<Action>`. Most command branches wrap their result with `add_change_mode()`, which prepends a `ChangeMode(Command, Normal/Navigation)` message so the app transitions out of Command mode after execution.

However, the `split` and `vsplit` commands have early-return error paths (`return vec![EmitMessages(Error(...))]`) and match-arm error paths that bypass `add_change_mode()`. This leaves the mode stuck in `Command(Command)` while the commandline buffer displays a red error message. Subsequent `:` keystrokes are interpreted as text insertions into the stale buffer instead of triggering a fresh command prompt.

## Goals / Non-Goals

**Goals:**
- Ensure all error paths in `split` and `vsplit` route through `add_change_mode()` so the mode always transitions back after command execution.

**Non-Goals:**
- Refactoring the entire `execute()` function to prevent similar future issues (though the fix pattern could inform future work).
- Changing the `PrintMultiline` mode handling (that path works correctly for multi-line errors).

## Decisions

**Restructure split/vsplit error handling to avoid early returns that bypass `add_change_mode()`.**

The `split` and `vsplit` match arms currently have this structure:
```
("split", args) => {
    match get_current_path(app) {
        Some(path) => {
            match file::expand_path(...) {
                Ok(target_path) => target_path,
                Err(err) => return vec![EmitMessages(Error(err))],  // bypasses
            };
            add_change_mode(...)
        }
        None => vec![EmitMessages(Error(...))],  // bypasses
    }
}
```

The fix wraps both error paths through `add_change_mode()`, either by restructuring the match to return errors as values passed to `add_change_mode()`, or by using `print_error()` (which already exists and calls `add_change_mode()`).

Alternative considered: Adding a mode-change guard at the end of `execute()` that checks if we're still in Command mode and forces a transition. Rejected because it would mask future bugs and adds implicit behavior.

## Risks / Trade-offs

- [Low risk] The fix is localized to 4 error paths in 2 commands. No behavioral change for successful command execution.
- [Trade-off] Using `print_error()` is the cleanest approach but changes the error path from `return` to normal control flow, which slightly changes the structure of the match arms.
