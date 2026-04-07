## Why

Yeet currently has no mechanism for users to extend functionality beyond what ships in the binary. A plugin system allows the community to share reusable extensions (custom hooks, commands, themes) distributed as git repositories, with reproducible version pinning via a lock file.

## What Changes

- New `yeet-plugin` crate providing plugin management (registration, sync, update, loading)
- New Lua table `y.plugin` exposing plugin management functions (`register`, `list`, etc.) called from user `init.lua`
- New commands `:pluginsync`, `:pluginupdate`, `:pluginlist` to control the plugin lifecycle from the command line
- Lock file (`plugins.lock`) created in the user config directory (`$XDG_CONFIG_HOME/yeet/`) tracking exact commit SHAs for reproducibility
- Plugin storage in `$XDG_DATA_HOME/yeet/plugins/` (or `~/.local/share/yeet/plugins/`)
- On startup, only already-downloaded plugins are loaded by executing their `init.lua`; missing plugins produce an error message listing what's missing (no auto-download)

## Capabilities

### New Capabilities

- `plugin-registry`: Lua API for declaring plugins via `y.plugin.register(opts)` in user config, including git endpoint, optional branch, and version/tag range constraints
- `plugin-lifecycle`: Sync and update operations — sync restores exact versions from lock file, update resolves latest allowed versions and writes new SHAs to lock file
- `plugin-loading`: Startup loading of already-downloaded plugins by executing each plugin's `init.lua`, allowing plugins to register hooks, themes, etc.; missing plugins are reported as errors without auto-downloading
- `plugin-commands`: User-facing commands (`:pluginsync`, `:pluginupdate`, `:pluginlist`) integrated into the existing command system

### Modified Capabilities

- `lua`: The `y` global table gains a new `y.plugin` sub-table with plugin management functions
- `commands`: New plugin-related commands added to the command dispatch

## Impact

- **New crate**: `yeet-plugin` added to workspace, depended on by `yeet-frontend` and `yeet-lua`
- **Dependencies**: Git operations require a git library (e.g., `git2` or shelling out to `git`)
- **File system**: New files written to XDG data and config directories (lock file, downloaded plugin sources)
- **Startup time**: Plugin loading adds I/O on startup; sync/update add network calls
- **Lua runtime**: Extended with `y.plugin` table; plugin `init.lua` files execute in the same runtime
