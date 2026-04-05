## Why

Help topic resolution via `:help <topic>` is case-sensitive. Users must type exact casing (e.g., `:help File Operations` works but `:help file operations` does not). This is unnecessarily strict — vim's `:help` is case-insensitive, and users shouldn't need to remember exact heading casing.

## What Changes

- Make all topic matching in `resolve_topic` case-insensitive: page names, page titles (`#`), section headings (`##`), and entry identifiers (`` ### `identifier` ``).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `help-command-dispatch`: Topic resolution SHALL match case-insensitively across all structural levels.

## Impact

- `yeet-frontend/src/update/command/help.rs`: Replace `==` comparisons in `resolve_topic` with `eq_ignore_ascii_case`.
