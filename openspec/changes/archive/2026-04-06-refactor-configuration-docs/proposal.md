## Why

The `docs/help/configuration.md` file has grown to cover three distinct concerns: general config file info, theme tokens, and hooks. As more hooks and tokens are added, a single file becomes harder to navigate. Splitting it improves discoverability and keeps each topic focused.

## What Changes

- Split `docs/help/configuration.md` into three files:
  - `configuration.md` — index page with Config File section (location, error handling) and links to theme and hooks docs
  - `theme.md` — the `y.theme` table, `syntax` setting, and all token sections (Tabbar, Statusline, Diff, Buffer, Border, Sign)
  - `hooks.md` — the `y.hook` table and all hook documentation (`on_window_create`)
- No code changes, no behavior changes — documentation restructuring only

## Capabilities

### New Capabilities

### Modified Capabilities

- `help`: The help system bundles docs at compile time from `docs/help/`. Adding new files and removing content from the existing file changes what `:help` topics resolve to.

## Impact

- **docs/help/configuration.md** — reduced to index with links
- **docs/help/theme.md** — new file with all theme content
- **docs/help/hooks.md** — new file with all hooks content
- **yeet-frontend/src/update/command/help.rs** — must register `theme.md` and `hooks.md` as help pages and update topic resolution
