## Why

The README shortcuts and commands tables are missing several implemented features. Users reading the README will not discover `:copen`, `gg`/`G` for jumping to top/bottom, or `Enter` for opening the selected entry in navigation mode.

## What Changes

- Add `:copen` to the commands table — opens a horizontal split with the quickfix list buffer (analogous to `:topen` for tasks)
- Add `gg`, `G` to the navigation and normal mode keybindings table — jump to top/bottom of the buffer
- Add `Enter` to the navigation mode keybindings table — open selected file/directory

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

_None. This is a documentation-only change to README.md._

## Impact

- `README.md` — three table sections updated
