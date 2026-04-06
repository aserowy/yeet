## 1. New help pages

- [x] 1.1 Create `docs/help/modes.md` with sections for mode transitions, Navigation mode (with all navigation-only keys: Enter, gh, gn, gt/gT, p, "p, yp, yy, C-n/C-p, C-w C-s/C-v), Normal mode (register targeting, all normal-only motions and edits), Insert mode, and Command mode. Every entry minimum two sentences.
- [x] 1.2 Create `docs/help/configuration.md` with sections for config file location (XDG paths, fallback), theme configuration (y.theme table, available tokens grouped by UI area), syntect theme selection (y.theme.syntax), and error handling for invalid config. Every entry minimum two sentences.

## 2. Enhance existing help pages

- [x] 2.1 Enhance `docs/help/keybindings.md`: Add all missing keybindings — shared navigation+normal keys (o, O, I, A, dd, space, zt/zz/zb), and all search/macro/mark keys already present get expanded to minimum two sentences each. Remove mode definitions section (moved to modes.md). Reorganize into: Navigation, Window Navigation, Cursor Movement (shared), Viewport, Marks, Search, Registers and Macros.
- [x] 2.2 Enhance `docs/help/commands.md`: Expand every single-sentence entry to minimum two sentences. Add context about constraints, related commands, default behavior, and error conditions where applicable.
- [x] 2.3 Enhance `docs/help/index.md`: Add entries for `modes` and `configuration` pages with two-sentence descriptions.

## 3. Register new help pages in Rust

- [x] 3.1 In `yeet-frontend/src/update/command/help.rs`, add `include_str!` constants and `HelpPage` entries for `modes.md` and `configuration.md`

## 4. Rework README.md

- [x] 4.1 Replace the keybinding tables (navigation mode, navigation+normal, normal mode sections) with a concise quick-start section (~10 essential keys) and link to `:help keybindings` and `:help modes` for the full reference
- [x] 4.2 Replace the commands table with a brief overview of command categories and link to `:help commands` for the full reference
- [x] 4.3 Shorten the configuration section to link to `:help configuration` for details, keeping only the config file path and a minimal example

## 5. Verify

- [x] 5.1 Run `markdownlint` on all `docs/` and `README.md` files and fix any warnings
- [x] 5.2 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
