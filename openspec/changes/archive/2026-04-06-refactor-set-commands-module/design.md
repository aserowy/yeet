## Context

The `:set` command was added with four match arms inlined in the main command dispatcher (`command/mod.rs`). Other command groups (qfix, help, split, task, print, file) each have their own submodule. As more `:set` options are added, the inline approach will not scale.

Currently the dispatcher has ~400 lines of match arms. The `:set` handling spans 4 arms (wrap, nowrap, empty, unknown). The `Window::set_wrap` method lives on the Window enum in `model/mod.rs`.

## Goals / Non-Goals

**Goals:**

- Extract all `:set` dispatch logic into `command/settings.rs`
- Reduce the `:set` handling in `mod.rs` to a single `("set", args)` delegation
- Keep `Window::set_wrap` on the Window type (it's a model method, not command logic)
- All existing tests pass unchanged

**Non-Goals:**

- Adding new `:set` options (that's a separate change)
- Changing any user-facing behavior
- Refactoring other command groups

## Decisions

**1. Module name: `settings` rather than `set`**

`set` is a Rust keyword-adjacent name (though not reserved, `std::collections::HashSet` etc.). `settings` is clearer about the module's purpose — it handles runtime settings commands. This follows the pattern where module names describe the domain (e.g., `split`, `file`, `task`) rather than the command verb.

Alternative: `set_cmd` or `set_command`. Rejected — inconsistent with existing module naming which uses domain nouns.

**2. Single public `execute` function**

The module exposes `pub fn execute(app: &mut App, args: &str, mode_before: Mode, mode: Mode) -> Vec<Action>` that handles argument parsing internally. This matches the delegation pattern used by other modules (e.g., `split::horizontal`, `qfix::commands::cdo`).

Alternative: Return just the actions without mode handling. Rejected — keeping mode transitions inside the module is consistent with how `print_error` works within the dispatcher.

**3. Keep Window::set_wrap on the model**

`set_wrap` is a model mutation — it changes viewport state. It belongs on Window, not in command dispatch code. The settings module calls `window.set_wrap()` rather than reaching into viewport fields directly.

## Risks / Trade-offs

- [Minimal risk] Pure refactoring with no behavior change. Existing command dispatch tests verify correctness.
