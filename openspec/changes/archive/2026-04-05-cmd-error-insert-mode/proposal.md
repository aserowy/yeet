## Why

When commands like `:split asdf` or `:vsplit badpath` fail, some error return paths in `command::execute()` bypass `add_change_mode()`, leaving the app stuck in `Command(Command)` mode while showing a red error message on the commandline. When the user then presses `:` to enter a new command, the keystroke is treated as a text insertion into the existing error buffer instead of starting a fresh command prompt. This results in `:<error message>` appearing on the commandline with the error text remaining red.

## What Changes

- Fix early-return error paths in `command::execute()` that bypass `add_change_mode()`, so the mode always transitions back to Normal/Navigation after command execution, even on error.
- The affected commands are `split` and `vsplit`, which have two error paths each (path expansion failure via `return`, and missing preview path via match arm) that skip the mode change.

## Capabilities

### New Capabilities

### Modified Capabilities

- `window-management`: Fix error handling in split/vsplit commands to always transition mode back after command execution, preventing the commandline from getting stuck in Command mode with stale error text.

## Impact

- `yeet-frontend/src/update/command/mod.rs`: The `split` and `vsplit` match arms in `execute()` need their error paths routed through `add_change_mode()`.
- No API or dependency changes.
- No breaking changes.
