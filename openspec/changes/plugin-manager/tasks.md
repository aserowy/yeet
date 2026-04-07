## 1. Crate Setup

- [x] 1.1 Create `yeet-plugin` crate with `Cargo.toml` (add `gix`, `toml`, `serde`, `semver`, `sha2`, `dirs`, `thiserror`, `tokio` as dependencies)
- [x] 1.2 Add `yeet-plugin` to workspace `Cargo.toml` members and `[workspace.dependencies]`
- [x] 1.3 Add `yeet-plugin` dependency to `yeet-frontend/Cargo.toml`
- [x] 1.4 Update `flake.nix` / nix build configuration for the new crate

## 2. Domain Types

- [x] 2.1 Define `PluginSpec` struct (url, branch, version, dependencies) in `yeet-plugin`
- [x] 2.2 Define `LockEntry` struct (commit, sha256, branch, tag) and `LockFile` struct with TOML serde support
- [x] 2.3 Define `PluginStatus` enum (`Loaded`, `Error`, `Missing`) and `PluginState` struct (url, status, error message, version/commit)
- [x] 2.4 Implement URL-to-storage-path derivation (`https://github.com/owner/repo` → `owner/repo`)

## 3. Path Resolution

- [x] 3.1 Implement `resolve_plugin_data_path()` using `$XDG_DATA_HOME/yeet/plugins/` with `~/.local/share/yeet/plugins/` fallback
- [x] 3.2 Implement `resolve_lock_file_path()` using `$XDG_CONFIG_HOME/yeet/plugins.lock` with `~/.config/yeet/plugins.lock` fallback
- [x] 3.3 Add tests for XDG path resolution with and without env vars set

## 4. Lock File Operations

- [x] 4.1 Implement lock file reading (parse TOML, return `LockFile` or empty if missing)
- [x] 4.2 Implement lock file writing (serialize `LockFile` to TOML, write atomically)
- [x] 4.3 Add tests for lock file round-trip (write then read), missing file handling, and corrupt file error

## 5. Version Resolution

- [x] 5.1 Implement semver range parsing from version constraint strings (e.g., `">=1.0, <2.0"`)
- [x] 5.2 Implement tag filtering: given a list of remote tags, filter by semver range and select latest match
- [x] 5.3 Implement dependency deduplication by URL with version constraint intersection
- [x] 5.4 Add tests for version resolution: matching tags, no match, no constraint, constraint intersection

## 6. Git Operations

- [x] 6.1 Implement shallow clone at a specific tag/commit using `gix`
- [x] 6.2 Implement clone at branch HEAD using `gix`
- [x] 6.3 Implement fetch and checkout to a specific commit for existing clones
- [x] 6.4 Implement listing remote tags from a repository
- [x] 6.5 Implement resolving remote HEAD branch name
- [x] 6.6 Implement SHA-256 tree hash computation for integrity verification
- [x] 6.7 Add tests for git operations (clone, fetch, checkout, tag listing) using local test repos

## 7. Sync Operation

- [x] 7.1 Implement sync logic: read lock file, for each registered plugin clone or checkout to locked commit, verify SHA-256
- [x] 7.2 Implement unregistered plugin cleanup: diff registered list against lock file and data directory, delete orphans
- [x] 7.3 Implement parallel execution using configured concurrency limit with tokio semaphore
- [x] 7.4 Implement error collection: gather per-plugin errors and return consolidated result
- [x] 7.5 Add tests for sync: all present, missing plugin cloned, cleanup of unregistered, integrity failure

## 8. Update Operation

- [x] 8.1 Implement update logic: for each registered plugin, fetch tags/branches, resolve latest allowed version, clone or checkout, compute SHA-256, write lock file
- [x] 8.2 Implement branch-only update path (no version constraint: checkout latest commit on branch)
- [x] 8.3 Implement unregistered plugin cleanup (reuse from sync)
- [x] 8.4 Implement parallel execution using configured concurrency limit
- [x] 8.5 Add tests for update: semver resolution, branch-only, no match error, lock file creation

## 9. Lua Integration — y.plugin Table

- [x] 9.1 Create `y.plugin` table in `setup_and_execute()` alongside `y.theme` and `y.hook`
- [x] 9.2 Implement `y.plugin.register(opts)` Lua function that validates opts and appends to an internal plugin list table
- [x] 9.3 Implement validation: reject missing `url`, non-table argument, warn on nested dependencies in dependency entries
- [x] 9.4 Implement `y.plugin.concurrency` as a settable field on the plugin table with default value 4
- [x] 9.5 Ensure `y.plugin` survives `y = { ... }` reassignment (same protection as `y.hook`)
- [x] 9.6 Implement `read_plugin_specs()` to read the plugin list from Lua and return `Vec<PluginSpec>`
- [x] 9.7 Implement `read_plugin_concurrency()` to read the concurrency setting from Lua
- [x] 9.8 Add tests: register with all opts, register URL-only, register with deps, missing URL, non-table arg, nested deps warning, concurrency read, y reassignment survival

## 10. Plugin Loading

- [x] 10.1 Implement plugin load ordering: topological sort dependencies before dependents, dedup shared dependencies
- [x] 10.2 Implement Lua state snapshot and rollback mechanism (save/restore hook tables, theme table state)
- [x] 10.3 Implement single plugin loading: locate `init.lua` in data dir, execute with snapshot/rollback on error
- [x] 10.4 Implement load orchestration: iterate ordered plugin list, track `PluginState` per plugin, skip missing, cascade dependency failures
- [x] 10.5 Implement missing plugin error aggregation: collect all missing plugins and emit single error message
- [x] 10.6 Add tests: successful load, missing plugin, syntax error rollback, runtime error rollback, dependency failure cascade, missing init.lua

## 11. Frontend Integration — Startup

- [x] 11.1 Call `read_plugin_specs()` after `init.lua` execution in startup flow (`yeet/src/lua.rs` or `yeet/src/main.rs`)
- [x] 11.2 Call plugin loading orchestration, passing Lua runtime and plugin specs
- [x] 11.3 Store `Vec<PluginState>` in `Model` (or alongside `LuaConfiguration`) for lifetime access
- [x] 11.4 Read `concurrency` setting and store in `Settings`

## 12. Frontend Integration — Commands

- [x] 12.1 Add `("pluginlist", "")` match arm in command dispatch, calling synchronous print handler (like `:marks`)
- [x] 12.2 Implement `pluginlist` print function: format each plugin's URL, status, and error message
- [x] 12.3 Add `("pluginsync", "")` match arm dispatching `Action::Task(Task::PluginSync(...))`
- [x] 12.4 Add `("pluginupdate", "")` match arm dispatching `Action::Task(Task::PluginUpdate(...))`
- [x] 12.5 Add `Task::PluginSync` and `Task::PluginUpdate` variants to the `Task` enum
- [x] 12.6 Implement async task handlers for sync and update, emitting `Message::Error` / `Message::Print` on completion
- [x] 12.7 Add tests for command dispatch: pluginlist output, pluginsync task creation, pluginupdate task creation

## 13. Documentation

- [x] 13.1 Add plugin manager section to `docs/` covering `y.plugin.register()`, commands, lock file, and plugin authoring
- [x] 13.2 Run `markdownlint` on all docs and fix any warnings
- [x] 13.3 Create `docs/help/plugins.md` help page with plugin registration, commands, lock file, authoring sections
- [x] 13.4 Add `plugins` to `HELP_PAGES` array and `include_str!` in `help.rs`
- [x] 13.5 Add `plugins` entry to `docs/help/index.md`
- [x] 13.6 Run `markdownlint` on new help page

## 14. Build Verification

- [x] 14.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 14.2 Run `cargo test` and ensure all tests pass
- [x] 14.3 Run `nix build .` and ensure build succeeds
