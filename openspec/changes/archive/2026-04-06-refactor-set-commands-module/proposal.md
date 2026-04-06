## Why

The `:set` command handling is inlined directly in the main command dispatcher (`command/mod.rs`) with four match arms. As more `:set` options are added, this will bloat the already large dispatcher. Other command groups (qfix, help, split, task) already have dedicated submodules. The `:set` commands should follow the same pattern.

## What Changes

- Create a new `settings` module at `yeet-frontend/src/update/command/settings.rs`
- Move all `:set` argument parsing and dispatch logic into the new module
- Move `Window::set_wrap` from `model/mod.rs` into the settings module as a helper or keep it on Window but call it from the new module
- Reduce the `:set` match arms in `command/mod.rs` to a single `("set", args)` arm that delegates to `settings::execute`

## Capabilities

### New Capabilities

None — this is a refactoring with no behavior changes.

### Modified Capabilities

None — no spec-level behavior changes.

## Impact

- `yeet-frontend/src/update/command/mod.rs`: Replace four `:set` match arms with single delegation
- `yeet-frontend/src/update/command/settings.rs`: New module containing set argument parsing and dispatch
- Existing tests must continue to pass with no changes to assertions
