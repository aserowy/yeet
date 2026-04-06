## Why

The help system covers all commands but only ~25% of keybindings. It has no documentation for modes or configuration. Meanwhile, README.md duplicates the full keybinding/command tables. Help entries are often a single terse sentence with no context. Users should be able to learn yeet entirely from `:help`.

## What Changes

- **New help page `modes.md`**: Documents all four modes (Navigation, Normal, Insert, Command), mode transition semantics (esc order, command mode exception), filesystem persistence rules (normal→navigation auto-saves), register targeting per mode (junk yard vs. default register).
- **New help page `configuration.md`**: Documents Lua config file location (XDG), the `y.theme` table, available theme tokens, syntect theme selection (`y.theme.syntax`), and error handling for invalid config.
- **Enhance `keybindings.md`**: Add all 40+ missing keybindings organized by mode. Every entry gets at minimum two sentences of description.
- **Enhance `commands.md`**: Expand every entry to at minimum two sentences. Add context about constraints, related commands, and behavior nuances.
- **Enhance `index.md`**: Add entries for the new `modes` and `configuration` pages.
- **Rework `README.md`**: Replace the detailed keybinding/command tables with concise summaries that link to `:help` topics. Keep the vision, screenshot, CLI usage, FAQ, and architecture overview. Remove duplicated reference content.
- **Register new help pages** in the `help::open` Rust code so `:help modes` and `:help configuration` work.
- **Run markdownlint** on all docs/ and README.md files after changes.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `help`: Help system adds new pages (modes, configuration), all entries expanded to minimum two sentences, all keybindings covered.

## Impact

- `docs/help/modes.md`: New file.
- `docs/help/configuration.md`: New file.
- `docs/help/keybindings.md`: Major expansion (40+ new entries).
- `docs/help/commands.md`: All entries expanded with detail.
- `docs/help/index.md`: New page links added.
- `README.md`: Keybinding/command tables replaced with links to help.
- `yeet-frontend/src/update/command/help.rs`: Register `modes.md` and `configuration.md` as help pages.
