## Context

`docs/help/configuration.md` is a single file containing three sections: Config File (location, error handling), Theme (y.theme, syntax, all token groups), and Hooks (y.hook, on_window_create). The help system in `help.rs` bundles markdown files at compile time via `include_str!` and registers them as `HelpPage` entries with a `name` field used for topic resolution. Currently `configuration` is one page covering everything.

## Goals / Non-Goals

**Goals:**

- Split `configuration.md` into `configuration.md` (index), `theme.md` (theme + all tokens), `hooks.md` (hooks)
- Register `theme` and `hooks` as new help pages in `help.rs` so `:help theme` and `:help hooks` work
- Keep `:help configuration` working (points to the index)
- Update help index to list the new pages

**Non-Goals:**

- Changing any documentation content beyond moving it between files
- Changing help system behavior or topic resolution logic

## Decisions

**File split:**
- `configuration.md` keeps: title, intro paragraph, Config File section (Location, Error handling), and links to theme.md and hooks.md
- `theme.md` gets: Theme section (y.theme, syntax) and all Token sections (Tabbar, Statusline, Diff, Buffer, Border, Sign)
- `hooks.md` gets: Hooks section (y.hook, on_window_create, context table, viewport settings table)

**Help page registration:** Add two new `include_str!` constants and `HelpPage` entries in `help.rs` for `theme` and `hooks`. The existing `configuration` entry stays and points to the trimmed index file.

**Index linking:** The trimmed `configuration.md` references the sub-pages by heading name so users can navigate via `:help theme` or `:help hooks`.

## Risks / Trade-offs

**Compile-time asset size** → Two more `include_str!` files. Negligible impact since total content is unchanged, just redistributed.
