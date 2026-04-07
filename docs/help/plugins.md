# Plugins

Yeet supports extending functionality through git-based plugins. Plugins are Lua scripts distributed as git repositories that can register hooks, themes, and other extensions.

## Registration

### `y.plugin.register`

Declare a plugin in your `init.lua`. The function takes a table with the following fields:

- `url` (string, required): Git repository URL
- `branch` (string, optional): Branch name, defaults to remote HEAD
- `version` (string, optional): Semver tag range (e.g. `">=1.0, <2.0"`)
- `dependencies` (table, optional): Array of dependency plugin specs

Example:

```lua
y.plugin.register({
    url = "https://github.com/user/yeet-theme-nord",
    branch = "main",
    version = ">=1.0, <2.0",
})
```

### `y.plugin.concurrency`

Set the number of parallel git operations during sync and update. Default is 4.

```lua
y.plugin.concurrency = 2
```

## Dependencies

Plugins can declare dependencies using the same options structure as `register()`:

```lua
y.plugin.register({
    url = "https://github.com/user/yeet-theme",
    dependencies = {
        { url = "https://github.com/user/yeet-lib-colors", version = ">=0.5" },
    },
})
```

Dependencies are loaded before the plugin that declares them. Shared dependencies are loaded once. Dependencies cannot declare sub-dependencies.

## Commands

### `pluginlist`

Show all registered plugins with their load status. Each entry displays the plugin URL and status (`loaded`, `error`, or `missing`). For plugins with errors, the error message is shown.

### `pluginsync`

Restore all registered plugins to the exact versions recorded in the lock file. If a plugin is not yet cloned, it is downloaded. Plugins removed from `init.lua` are deleted from disk and the lock file. Requires a lock file to exist — run `:pluginupdate` first.

### `pluginupdate`

Fetch the latest allowed versions for all registered plugins and update the lock file. For plugins with a `version` constraint, the latest matching semver tag is selected. For plugins without constraints, the latest commit on the configured branch is used. Plugins removed from `init.lua` are cleaned up.

## Lock File

The lock file is stored at `$XDG_CONFIG_HOME/yeet/plugins.lock` (or `~/.config/yeet/plugins.lock`). It is a TOML file recording the exact commit SHA and tree hash for each plugin. Version-control this file with your dotfiles for reproducible setups across machines.

## Plugin Storage

Downloaded plugins are stored in `$XDG_DATA_HOME/yeet/plugins/<owner>/<repo>/` (or `~/.local/share/yeet/plugins/<owner>/<repo>/`).

## Startup Behavior

On startup, yeet loads all registered plugins that are already downloaded by executing each plugin's `init.lua` in registration order. Missing plugins are not auto-downloaded — an error message lists what is missing.

If a plugin's `init.lua` fails, the Lua state is rolled back so no partial side effects persist. The plugin is marked as `error` and other plugins continue loading. Use `:pluginlist` to inspect errors.

## Writing Plugins

A yeet plugin is a git repository with an `init.lua` at its root. The script has access to the full `y` table and can register hooks, set theme colors, and use any Lua APIs available in the yeet runtime.

Example plugin `init.lua`:

```lua
y.theme.StatusLineFg = '#e0e0e0'

y.hook.on_window_create:add(function(ctx)
    if ctx.type == "directory" then
        ctx.preview.wrap = true
    end
end)
```
