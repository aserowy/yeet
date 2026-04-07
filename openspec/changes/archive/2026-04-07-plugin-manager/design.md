## Context

Yeet is a TUI file manager built as a Rust workspace with 5 crates. User configuration is done through `init.lua` loaded from `$XDG_CONFIG_HOME/yeet/`. The Lua runtime exposes a protected `y` global table with sub-tables for theming (`y.theme`) and hooks (`y.hook`). There is no extension mechanism beyond what ships in the binary.

The plugin manager introduces a git-based plugin system where plugins are declared in the user's `init.lua`, stored locally as cloned repositories, and loaded at startup by executing each plugin's `init.lua`.

## Goals / Non-Goals

**Goals:**

- Users can declare plugins in `init.lua` via `y.plugin.register(opts)` with git endpoint, optional branch, and version/tag range
- `:pluginsync` restores exact versions from a `plugins.lock` file
- `:pluginupdate` resolves latest allowed versions per configured constraints and updates the lock file
- `:pluginlist` shows installed plugins and their status
- On startup, already-downloaded plugins are loaded; missing plugins produce an error listing what's absent
- Lock file enables reproducible setups across machines

**Non-Goals:**

- Plugin registry or discovery service
- Auto-downloading plugins on startup
- Transitive dependency resolution beyond one level (dependencies of dependencies are not automatically resolved)
- Sandboxing plugin Lua execution beyond what the existing runtime provides
- Plugin uninstall command (users can remove the register call and delete the directory)

## Decisions

### 1. New `yeet-plugin` crate

**Decision**: Create a dedicated `yeet-plugin` crate for all plugin management logic.

**Rationale**: Keeps plugin concerns (git operations, lock file parsing, version resolution) isolated from the Lua and frontend crates. The crate owns the domain types (`PluginSpec`, `LockEntry`) and operations (clone, fetch, checkout, lock file read/write).

**Alternatives considered**:
- Putting plugin logic in `yeet-lua` — rejected because git/filesystem operations don't belong in the Lua integration layer
- Putting it in `yeet-frontend` — rejected because plugin management is not a UI concern

### 2. Git operations via `gix` (gitoxide)

**Decision**: Use the `gix` crate for clone, fetch, and checkout operations.

**Rationale**: Pure Rust implementation — no C FFI, so it works with the workspace-wide `unsafe_code = "forbid"` lint without exceptions. No external `git` binary required. Supports shallow clones for faster initial downloads.

**Alternatives considered**:
- `git2` (libgit2 bindings) — rejected because it uses C FFI and would require relaxing `unsafe_code = "forbid"` in the new crate
- Shelling out to `git` CLI — rejected because it adds an external dependency and complicates error handling

### 3. Plugin storage layout

**Decision**: Store plugins in `$XDG_DATA_HOME/yeet/plugins/<owner>/<repo>/` (falling back to `~/.local/share/yeet/plugins/`).

Each plugin directory is a git clone. The directory structure uses `<owner>/<repo>` derived from the git URL to avoid name collisions.

**Rationale**: XDG data home is the correct location for application data that is not configuration. Separating by owner/repo avoids collisions when different authors name plugins similarly.

### 4. Lock file format and location

**Decision**: Use a TOML lock file at `$XDG_CONFIG_HOME/yeet/plugins.lock`.

```toml
[plugins."github.com/user/plugin-name"]
commit = "abc123def456"
sha256 = "..."
branch = "main"
tag = "v1.2.0"
```

**Rationale**: TOML is already familiar in the Rust ecosystem. Storing in the config directory (alongside `init.lua`) means the lock file can be version-controlled with the user's dotfiles for reproducibility. The SHA-256 hash of the checkout tree provides integrity verification.

**Alternatives considered**:
- JSON — less readable, no comments
- Placing in data directory — wouldn't travel with dotfiles

### 5. Plugin registration as Lua-side declarations

**Decision**: Plugins are declared in the user's `init.lua` via `y.plugin.register()`. This builds an in-memory plugin list that the Rust side reads after config evaluation.

```lua
y.plugin.register({
    url = "https://github.com/user/yeet-theme-nord",
    branch = "main",           -- optional, defaults to remote HEAD
    version = ">=1.0, <2.0",   -- optional semver tag range
    dependencies = {            -- optional, same opts structure
        {
            url = "https://github.com/user/yeet-lib-colors",
            version = ">=0.5",
        },
    },
})
```

Dependencies use the same opts shape as `register()` itself. They are synced/updated alongside the parent plugin. During loading, dependencies are loaded before the plugin that declares them. If a dependency is declared by multiple plugins, it is loaded once (deduplication by URL). If a dependency fails to load, the dependent plugin is also marked as `error`.

**Rationale**: Follows the existing pattern where `init.lua` declares configuration (`y.theme`, `y.hook`) and Rust reads it back. Users get a single configuration file. The register call is purely declarative — it stores the spec, no network calls happen during config evaluation. Declaring dependencies inline keeps the plugin graph visible in a single place.

### 6. Version resolution strategy

**Decision**: Version constraints use semver tag ranges. `:pluginupdate` fetches remote tags, filters by the configured range, selects the latest matching tag, and records the tagged commit SHA in the lock file.

If no version constraint is set, update checks out the latest commit on the configured branch (or default branch).

**Rationale**: Semver ranges give users control over breaking changes. Tag-based resolution is simple and doesn't require plugins to maintain a manifest.

### 6b. Unregistered plugin cleanup

**Decision**: The plugin list from `init.lua` is the source of truth. On `:pluginupdate` or `:pluginsync`, any plugins present in the lock file or data directory but no longer registered in `init.lua` are deleted from the data directory and removed from the lock file.

**Rationale**: Keeps the lock file and cache in sync with the user's declared intent. Removing the `register()` call is the natural way to "uninstall" a plugin — no separate uninstall command needed. Cleanup happens only on explicit user action (sync/update), never silently on startup.

### 7. Plugin loading order and error handling

**Decision**: On startup, after `init.lua` executes and the plugin list is built, each registered plugin's `init.lua` is executed in registration order. The runtime maintains a `PluginState` per plugin tracking status (`loaded`, `error`, `missing`) and an optional error message.

**Missing plugins**: If a plugin directory doesn't exist, it is recorded as `missing` and skipped. After all present plugins are attempted, missing plugins are reported as a single error message.

**Failed plugins**: Each plugin's `init.lua` is executed against a snapshot of the Lua state. If execution errors (syntax error, runtime error, etc.), the Lua state is rolled back to the pre-execution snapshot so no partial side effects (half-registered hooks, incomplete theme overrides) persist. The plugin is recorded as `error` with the error message. Loading continues with the next plugin.

**State in memory**: The per-plugin state (status + error message) is kept in memory for the lifetime of the application. `:pluginlist` displays each plugin's name, version, and status — including the error message for failed or missing plugins.

**Rationale**: Rolling back on failure ensures a failed plugin doesn't leave the application in an inconsistent state with partial hooks or theme entries. Persisting error state in memory gives users a way to diagnose issues via `:pluginlist` without checking logs. Loading in registration order gives deterministic behavior. Continuing past failures ensures one broken plugin doesn't block the application.

### 8. Command integration

**Decision**: Add `:pluginsync`, `:pluginupdate`, and `:pluginlist` to the existing command dispatch in `yeet-frontend/src/update/command/mod.rs`. `:pluginsync` and `:pluginupdate` dispatch async tasks since they involve network I/O. `:pluginlist` is synchronous — it prints the registered plugins and their loading status from in-memory state (similar to `:marks` or `:junk`). All output (success messages, errors, cleanup reports) is printed to the command line via `PrintError`/print actions, not the tasks window.

**Rationale**: Sync and update involve network calls and belong in the async task system. List only reads already-available in-memory data (registration + load state), so there's no reason to make it async. Consolidating output in the command line keeps plugin feedback consistent and visible without requiring the user to open the tasks window.

### 9. Dependency graph

```
yeet (binary)
  └─ yeet-frontend
       ├─ yeet-lua
       ├─ yeet-keymap
       ├─ yeet-buffer
       └─ yeet-plugin
            └─ gix

yeet-lua (config loading)
  └─ yeet-buffer
```

`yeet-plugin` is depended on by `yeet-frontend` only. The Lua registration API lives in `yeet-lua` (it just builds a data structure), while `yeet-plugin` handles git operations and lock file management. `yeet-frontend` coordinates between them: reads the plugin list from Lua, passes it to `yeet-plugin` for sync/update/load operations.

## Risks / Trade-offs

**Network operations block the UI during sync/update** → Run sync and update as async tasks (like `:fd`), showing progress in the tasks window. Loading at startup is local I/O only (no network). Parallelism is configurable (see decision 10).

### 10. Configurable parallel git operations

**Decision**: The number of concurrent clone/fetch operations during sync and update is configurable via `y.plugin.concurrency` in Lua (and exposed as a field in `Settings`). Defaults to 4.

```lua
y.plugin.concurrency = 2
```

**Rationale**: Users on slow connections or constrained systems may want to limit parallel network operations. Power users may want to increase it. Exposing it in Lua keeps it consistent with the existing configuration model.

**Large plugin repositories slow down initial clone** → Support shallow clones (`--depth 1` equivalent) when cloning at a specific tag/commit.

**Plugin `init.lua` can do anything in the Lua runtime** → Accepted trade-off. Same trust model as the user's own `init.lua`. Sandboxing is a non-goal for now.

**Lock file conflicts in shared dotfiles** → TOML format with one entry per plugin key minimizes merge conflicts. Users can resolve manually like any config file.
