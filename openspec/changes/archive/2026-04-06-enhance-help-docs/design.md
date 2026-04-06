## Context

The help system uses embedded markdown files loaded via `include_str!` in `help.rs`. Adding new pages requires: creating the markdown file, adding a `const` with `include_str!`, and adding a `HelpPage` entry to the `HELP_PAGES` array. Topic resolution already handles all heading levels, so new pages are automatically searchable.

The README currently contains ~100 lines of keybinding tables and ~40 lines of command tables that duplicate help content. The README should focus on quick-start, vision, and links rather than being a reference manual.

## Goals / Non-Goals

**Goals:**

- Every yeet feature documented in at least one help page.
- Every help entry has minimum two sentences of description.
- New help pages for modes and configuration.
- README links to help topics instead of duplicating tables.
- All docs pass markdownlint.

**Non-Goals:**

- Changing any application behavior.
- Adding new features or commands.
- Restructuring the help system architecture.

## Decisions

**Help page structure for `modes.md`**

Organized by mode with `##` sections for each mode (Navigation, Normal, Insert, Command). Each mode section explains: what the mode is for, how to enter/exit it, register targeting, and key behaviors specific to that mode. A top-level section explains mode transition semantics and filesystem persistence.

**Help page structure for `configuration.md`**

Sections for: config file location (XDG paths), theme configuration (`y.theme` table), available theme tokens (grouped by UI area), syntect theme selection, and error handling.

**Keybindings.md reorganization**

Expand from the current 4 sections to cover all modes completely. Add sections for: Navigation-only keys (Enter, gh, gn, gt/gT, p, yp, yy, C-n/C-p, C-w C-s/C-v), Shared navigation+normal keys (j, k, gg, G, o, O, I, A, dd, :, /, ?, n, N, space, q/macro, @, m, ', zt/zz/zb, C-u/C-d), Normal-only motion keys (h, l, 0, $, f, F, t, T, ;, ,, e, E, ge, gE, w, W, b, B), Normal-only edit keys (i, a, c, d, s, x, .).

**README rework approach**

Keep: vision, screenshot, CLI, configuration section (shortened to link to `:help configuration`), FAQ, architecture. Replace keybinding/command tables with a brief "quick start" section listing ~10 essential keys and a pointer to `:help keybindings` and `:help commands` for the full reference.

## Risks / Trade-offs

- [README becomes less self-contained] → Acceptable. Users who install yeet can use `:help`. README serves as introduction and quick-start, not a reference manual.
- [Help pages grow large] → The keybindings page will be the largest. The structured heading format (### `key`) keeps it navigable via `:help <key>`.
