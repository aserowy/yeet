## Why

Word wrap rendering is fully implemented in the view layer, but there is no way for users to toggle it at runtime. Users need `:set wrap` and `:set nowrap` commands to control word wrapping per-viewport without restarting the application.

## What Changes

- Add `:set wrap` command that enables word wrapping on the focused viewport
- Add `:set nowrap` command that disables word wrapping on the focused viewport
- For Directory windows, `:set wrap` and `:set nowrap` apply to all three viewports (parent, current, preview)
- For single-viewport windows (QuickFix, Tasks, Help), the command applies to that single viewport

## Capabilities

### New Capabilities

- `commands`: Runtime `:set` command infrastructure for toggling viewport options

### Modified Capabilities

- `buffer`: Add wrap toggle behavior triggered by the new `:set wrap`/`:set nowrap` commands

## Impact

- `yeet-frontend/src/update/command/mod.rs`: Add `:set` command matching and dispatch
- `yeet-frontend/src/model/mod.rs`: Add method to set wrap on all viewports of a Window
- No dependency changes
- No breaking changes
