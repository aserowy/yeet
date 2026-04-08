# Yeet Help

Welcome to the yeet help system. Use `:help <topic>` to jump to a specific topic. Topic search is case-insensitive and matches page names, section headings, and entry identifiers.

## Help Pages

### `commands`

Reference for all available commands entered via `:` in command mode. Each command includes usage syntax, constraints, and related commands. See `:help commands`.

### `keybindings`

Quick reference of all keybindings organized by category. Covers navigation, window management, file operations, search, macros, and more. See `:help keybindings`.

### `modes`

Detailed documentation of yeet's four modes: Navigation, Normal, Insert, and Command. Covers mode transitions, filesystem persistence rules, register targeting, and all mode-specific keybindings. See `:help modes`.

### `configuration`

Guide to customizing yeet via the Lua configuration file. Covers config file location and error handling. See `:help configuration`.

### `theme`

Theme color tokens and syntax highlighting theme selection. Covers the `y.theme` table and all available color tokens for tabbar, statusline, buffer, borders, and signs. See `:help theme`.

### `hooks`

Lifecycle hooks for customizing yeet behavior via Lua callbacks. Covers the `y.hook` table and available hooks like `on_window_create`. See `:help hooks`.

### `plugins`

Plugin manager for extending yeet with git-based plugins. Covers `y.plugin.register()`, plugin commands (`:pluginlist`, `:pluginsync`, `:pluginupdate`), the lock file, and plugin authoring. See `:help plugins`.

### `help`

The help system itself. Use `:help` to open this index page, `:help <topic>` to jump to a specific topic, and `:q` to close the help window.

Usage:

- `:help` opens this index page
- `:help <topic>` opens the help page for the given topic
- `:q` closes the help window
