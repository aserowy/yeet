## Why

When opening help to a specific topic via `:help <topic>`, the matched heading is scrolled into view but may appear anywhere in the viewport (middle, bottom, or top depending on buffer length). This makes the help experience inconsistent — the user expects to see the heading at the top with its content below, similar to how `zt` positions the cursor line at the viewport top in vim.

## What Changes

- When `:help <topic>` resolves to a specific line offset, apply `zt`-equivalent viewport positioning so the matched heading appears at the top of the help buffer viewport.
- No change to bare `:help` (index page opens at line 0, which is already at the top).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `help-command-dispatch`: When a topic resolves to a non-zero line offset, the viewport SHALL be positioned so the matched heading is at the top of the visible area.

## Impact

- `yeet-frontend/src/update/command/help.rs`: After setting cursor position, also set `viewport.vertical_index` to match cursor line (zt behavior).
