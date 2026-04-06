## 1. Create new doc files

- [x] 1.1 Create `docs/help/theme.md` with the Theme section (`y.theme`, `syntax`) and all Token sections (Tabbar, Statusline, Diff, Buffer, Border, Sign) extracted from `configuration.md`
- [x] 1.2 Create `docs/help/hooks.md` with the Hooks section (`y.hook`, `on_window_create`, context table, viewport settings table) extracted from `configuration.md`
- [x] 1.3 Trim `docs/help/configuration.md` to keep only the title, intro, Config File section, and add references to `:help theme` and `:help hooks`

## 2. Register help pages

- [x] 2.1 Add `include_str!` constants for `theme.md` and `hooks.md` in `help.rs`
- [x] 2.2 Add `HelpPage` entries for `theme` and `hooks` in the `HELP_PAGES` array

## 3. Update help index

- [x] 3.1 Add entries for `theme` and `hooks` pages in `docs/help/index.md`

## 4. Validation

- [x] 4.1 Run `markdownlint` on `configuration.md`, `theme.md`, `hooks.md`, and `index.md` and fix any issues
- [x] 4.2 Run `cargo test` to verify help system tests pass
- [x] 4.3 Run `cargo clippy` and `cargo fmt` and fix any issues
