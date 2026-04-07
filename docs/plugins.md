# Plugin Manager

Yeet supports extending functionality through git-based plugins. Plugins are
Lua scripts distributed as git repositories that can register hooks, themes,
and other extensions.

## Registering Plugins

Plugins are declared in your `init.lua` using `y.plugin.register()`:

```lua
y.plugin.register({
    url = "https://github.com/user/yeet-theme-nord",
    branch = "main",           -- optional, defaults to remote HEAD
    version = ">=1.0, <2.0",   -- optional semver tag range
})
```

### Options

| Field          | Type   | Required | Description                          |
|----------------|--------|----------|--------------------------------------|
| `url`          | string | yes      | Git repository URL                   |
| `branch`       | string | no       | Branch name (defaults to remote HEAD)|
| `version`      | string | no       | Semver tag range (e.g. `">=1.0"`)    |
| `dependencies` | table  | no       | Array of dependency plugin specs     |

### Dependencies

Plugins can declare dependencies using the same options structure:

```lua
y.plugin.register({
    url = "https://github.com/user/yeet-theme",
    dependencies = {
        { url = "https://github.com/user/yeet-lib-colors", version = ">=0.5" },
    },
})
```

Dependencies are loaded before the plugin that declares them. If multiple
plugins share a dependency, it is loaded once. Dependencies cannot declare
their own sub-dependencies.

## Concurrency

Configure the number of parallel git operations during sync and update:

```lua
y.plugin.concurrency = 2  -- default is 4
```

## Commands

| Command          | Description                                           |
|------------------|-------------------------------------------------------|
| `:pluginlist`    | Show registered plugins and their load status         |
| `:pluginsync`    | Restore plugins to versions in the lock file          |
| `:pluginupdate`  | Fetch latest allowed versions and update the lock file|

## Lock File

Running `:pluginupdate` creates `plugins.lock` in your config directory
(`$XDG_CONFIG_HOME/yeet/plugins.lock`). This TOML file records the exact
commit SHA and tree hash for each plugin. You can version-control this file
with your dotfiles for reproducible setups.

Running `:pluginsync` restores plugins to the exact versions in the lock file.

## Plugin Storage

Downloaded plugins are stored in `$XDG_DATA_HOME/yeet/plugins/` (or
`~/.local/share/yeet/plugins/`), organized as `<owner>/<repo>/`.

## Startup Behavior

On startup, yeet loads all registered plugins that are already downloaded by
executing each plugin's `init.lua`. Missing plugins are not auto-downloaded;
instead, an error message lists what is missing.

If a plugin's `init.lua` fails (syntax error, runtime error), the Lua state
is rolled back so no partial side effects persist. The plugin is marked as
`error` and other plugins continue loading.

Use `:pluginlist` to see the status of all plugins, including error messages.

## Cleanup

Removing a `register()` call from `init.lua` and then running `:pluginsync`
or `:pluginupdate` will delete the plugin from disk and remove it from the
lock file.

## Writing Plugins

A yeet plugin is a git repository with an `init.lua` at its root. The
`init.lua` has access to the full `y` table and can register hooks, set
theme colors, and use any Lua APIs available in the yeet runtime.

Example plugin `init.lua`:

```lua
y.theme.StatusLineFg = '#e0e0e0'

y.hook.on_window_create:add(function(ctx)
    if ctx.type == "directory" then
        ctx.preview.wrap = true
    end
end)
```
