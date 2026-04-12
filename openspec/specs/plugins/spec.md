## Registration

### Requirement: Plugin registration via y.plugin.register()

The system SHALL expose a `y.plugin.register(opts)` function in the Lua environment. The `opts` table SHALL accept the following fields:

- `url` (string, required): Git repository URL
- `name` (string, optional): Override the plugin's `require()` name. Defaults to the last URL path segment (with `.git` suffix stripped).
- `branch` (string, optional): Branch name, defaults to remote HEAD
- `version` (string, optional): Semver tag range constraint (e.g., `">=1.0, <2.0"`)
- `dependencies` (table, optional): Array of dependency tables using the same opts shape (without nested dependencies)

Each call to `register()` SHALL append the plugin specification to an in-memory list. No network calls SHALL occur during registration.

#### Scenario: Register a plugin with all options

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-nord", branch = "main", version = ">=1.0, <2.0" })`
- **THEN** the plugin specification SHALL be stored in memory with the given URL, branch, and version constraint

#### Scenario: Register a plugin with only URL

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-nord" })`
- **THEN** the plugin specification SHALL be stored with branch defaulting to remote HEAD and no version constraint

#### Scenario: Register a plugin with dependencies

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-theme", dependencies = { { url = "https://github.com/user/yeet-lib" } } })`
- **THEN** the plugin specification SHALL be stored with one dependency entry

#### Scenario: Register called without URL

- **WHEN** `init.lua` contains `y.plugin.register({ branch = "main" })`
- **THEN** the system SHALL log an error and not add the entry to the plugin list

#### Scenario: Register called with non-table argument

- **WHEN** `init.lua` contains `y.plugin.register("https://github.com/user/plugin")`
- **THEN** the system SHALL log an error and not add the entry to the plugin list

#### Scenario: Register with explicit name

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-theme", name = "theme" })`
- **THEN** the plugin SHALL be accessible via `require('theme')`

#### Scenario: Register without name uses URL segment

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-theme" })`
- **THEN** the plugin SHALL be accessible via `require('yeet-theme')`

#### Scenario: Register with non-HTTPS URL is rejected

- **WHEN** `init.lua` contains `y.plugin.register({ url = "git@github.com:user/repo.git" })`
- **THEN** the system SHALL log an error indicating only HTTPS URLs are supported and SHALL NOT add the entry to the plugin list

#### Scenario: Register with HTTP URL is rejected

- **WHEN** `init.lua` contains `y.plugin.register({ url = "http://github.com/user/repo" })`
- **THEN** the system SHALL log an error indicating only HTTPS URLs are supported and SHALL NOT add the entry to the plugin list

### Requirement: Plugin list is readable from Rust

The system SHALL provide a function to read the registered plugin list from the Lua runtime as a Vec of plugin specification structs. This list SHALL include all plugins and their dependencies.

#### Scenario: Reading plugins after registration

- **WHEN** two plugins have been registered via `y.plugin.register()` and Rust reads the plugin list
- **THEN** the returned list SHALL contain two entries in registration order with their respective fields

#### Scenario: Reading plugins when none registered

- **WHEN** no plugins have been registered and Rust reads the plugin list
- **THEN** the returned list SHALL be empty

### Requirement: Dependency deduplication by URL

When multiple plugins declare the same dependency URL, the system SHALL treat them as a single dependency. If version constraints differ, the system SHALL use the most restrictive intersection.

#### Scenario: Same dependency declared by two plugins

- **WHEN** plugin A declares dependency `{ url = "https://github.com/user/lib", version = ">=1.0" }` and plugin B declares dependency `{ url = "https://github.com/user/lib", version = "<2.0" }`
- **THEN** the system SHALL resolve a single dependency with constraint `">=1.0, <2.0"`

#### Scenario: Duplicate dependency with identical opts

- **WHEN** plugin A and plugin B both declare dependency `{ url = "https://github.com/user/lib" }`
- **THEN** the system SHALL resolve a single dependency entry

### Requirement: Dependencies cannot have sub-dependencies

The system SHALL ignore any `dependencies` field within a dependency entry. Only top-level plugins may declare dependencies.

#### Scenario: Dependency with nested dependencies

- **WHEN** a plugin registers with `dependencies = { { url = "...", dependencies = { { url = "..." } } } }`
- **THEN** the system SHALL log a warning and ignore the nested `dependencies` field, registering only the direct dependency

### Requirement: Concurrency setting via y.plugin.concurrency

The system SHALL expose `y.plugin.concurrency` as a configurable integer on the `y.plugin` table. The default value SHALL be 4. This value controls the maximum number of parallel git operations during sync and update.

#### Scenario: Setting concurrency

- **WHEN** `init.lua` contains `y.plugin.concurrency = 2`
- **THEN** the system SHALL use 2 as the maximum parallel git operations

#### Scenario: Default concurrency

- **WHEN** `init.lua` does not set `y.plugin.concurrency`
- **THEN** the system SHALL use 4 as the default

#### Scenario: Invalid concurrency value

- **WHEN** `init.lua` contains `y.plugin.concurrency = "fast"`
- **THEN** the system SHALL log a warning and use the default value of 4

## Loading

### Requirement: Plugins are loaded on startup after init.lua

The system SHALL load plugins after `init.lua` has been executed and the plugin registration list has been built. Loading SHALL occur before UI rendering begins.

#### Scenario: Plugins loaded before first render

- **WHEN** yeet starts with registered plugins that are all present on disk
- **THEN** all plugin `init.lua` files SHALL be executed before the first UI render

#### Scenario: No plugins registered

- **WHEN** yeet starts and no plugins have been registered via `y.plugin.register()`
- **THEN** the system SHALL skip the plugin loading phase and proceed to UI rendering

### Requirement: Plugin loading executes init.lua

For each registered plugin, the system SHALL look for an `init.lua` file in the plugin's data directory and execute it in the Lua runtime. Plugins SHALL be loaded in registration order.

#### Scenario: Plugin with init.lua

- **WHEN** plugin A is registered and its directory contains `init.lua`
- **THEN** the system SHALL execute `init.lua` in the Lua runtime

#### Scenario: Plugin directory exists but has no init.lua

- **WHEN** plugin A is registered and its directory exists but contains no `init.lua`
- **THEN** the system SHALL record the plugin as `error` with a message indicating missing `init.lua`

### Requirement: Dependencies are loaded before dependents

The system SHALL load plugin dependencies before the plugin that declares them. If dependency B is required by plugin A, B's `init.lua` SHALL execute before A's `init.lua`.

#### Scenario: Plugin with one dependency

- **WHEN** plugin A depends on library B and both are present on disk
- **THEN** library B's `init.lua` SHALL execute before plugin A's `init.lua`

#### Scenario: Shared dependency loaded once

- **WHEN** plugin A and plugin B both depend on library C
- **THEN** library C's `init.lua` SHALL execute exactly once, before whichever dependent is loaded first

### Requirement: Only downloaded plugins are loaded

The system SHALL only attempt to load plugins whose directories exist in the data directory. Missing plugins SHALL NOT trigger any download or network operation.

#### Scenario: Plugin directory missing

- **WHEN** plugin A is registered but its directory does not exist in the data folder
- **THEN** the system SHALL record plugin A as `missing` and skip loading it

#### Scenario: Some plugins present, some missing

- **WHEN** plugins A and B are registered, A exists on disk but B does not
- **THEN** the system SHALL load A and record B as `missing`

### Requirement: Missing plugins reported as error on startup

After the plugin loading phase, if any registered plugins are missing from disk, the system SHALL emit a single error message listing all missing plugins.

#### Scenario: Two missing plugins

- **WHEN** plugins A and B are both registered but neither directory exists
- **THEN** the system SHALL emit one error message listing both A and B as missing

#### Scenario: All plugins present

- **WHEN** all registered plugins exist on disk
- **THEN** the system SHALL NOT emit any missing plugin error

### Requirement: Failed plugin init.lua is rolled back

If a plugin's `init.lua` raises an error during execution, the system SHALL roll back the Lua state to a snapshot taken before that plugin's execution. No partial side effects (hooks, theme changes) from the failed plugin SHALL persist.

#### Scenario: Plugin with syntax error

- **WHEN** plugin A's `init.lua` contains a syntax error
- **THEN** the Lua state SHALL be rolled back to before plugin A's execution, and plugin A SHALL be recorded as `error` with the error message

#### Scenario: Plugin with runtime error after registering a hook

- **WHEN** plugin A's `init.lua` registers a hook via `y.hook.on_window_create:add(fn)` and then raises a runtime error
- **THEN** the hook registration SHALL be rolled back and the hook list SHALL not contain the function from plugin A

#### Scenario: Failed plugin does not block others

- **WHEN** plugin A fails during loading and plugin B is next in registration order
- **THEN** plugin B's `init.lua` SHALL still be executed

### Requirement: Failed dependency marks dependent as error

If a dependency fails to load (error or missing), any plugin that depends on it SHALL also be marked as `error` with a message indicating the failed dependency. The dependent plugin's `init.lua` SHALL NOT be executed.

#### Scenario: Dependency fails, dependent skipped

- **WHEN** dependency B fails to load and plugin A depends on B
- **THEN** plugin A SHALL be recorded as `error` with a message referencing B's failure, and A's `init.lua` SHALL NOT execute

#### Scenario: Dependency missing, dependent skipped

- **WHEN** dependency B is missing from disk and plugin A depends on B
- **THEN** plugin A SHALL be recorded as `error` with a message referencing B being missing

### Requirement: Per-plugin state tracking

The system SHALL maintain a `PluginState` for each registered plugin in memory for the application lifetime. Each state SHALL contain:

- Plugin name/URL
- Status: one of `loaded`, `error`, `missing`
- Error message (if status is `error` or `missing`)
- Resolved version/commit (if known)

#### Scenario: Plugin loaded successfully

- **WHEN** plugin A loads without error
- **THEN** its state SHALL be `loaded` with no error message

#### Scenario: Plugin failed to load

- **WHEN** plugin A's `init.lua` raises an error
- **THEN** its state SHALL be `error` with the error message from the Lua runtime

#### Scenario: Plugin missing from disk

- **WHEN** plugin A's directory does not exist
- **THEN** its state SHALL be `missing` with a message indicating the expected path

### Requirement: Plugin loading supports require()

The system SHALL add each plugin's directory to Lua's `package.path` before executing the plugin's `init.lua`. If the plugin's `init.lua` returns a non-nil value, the system SHALL store it in `package.loaded` under the plugin's name. This enables Lua's standard `require()` to find and return plugin modules. The plugin name defaults to the last URL path segment; it can be overridden with the `name` field in `register()`.

#### Scenario: Plugin returns a module table

- **WHEN** a plugin's `init.lua` returns a table with a `setup` function
- **THEN** the returned table SHALL be stored in `package.loaded` under the plugin's name and be accessible via `require('plugin-name')`

#### Scenario: Plugin returns nil

- **WHEN** a plugin's `init.lua` does not return a value
- **THEN** `package.loaded` SHALL not be modified for that plugin and the plugin SHALL still be considered loaded

#### Scenario: Plugin name derived from URL

- **WHEN** a plugin is registered with URL `https://github.com/aserowy/yeet-bluloco-theme`
- **THEN** the derived plugin name for `require()` SHALL be `yeet-bluloco-theme` (last URL segment, no prefix stripping)

#### Scenario: require() loads plugin from disk when available

- **WHEN** a plugin is registered and its directory exists on disk with `init.lua`, and `require('plugin-name')` is called during `init.lua`
- **THEN** the searcher SHALL load the plugin's `init.lua` via `dofile()`, store the result in `package.loaded`, and return the real module table

#### Scenario: require() returns proxy when plugin not on disk

- **WHEN** a plugin is registered but its directory does not exist on disk (fresh install), and `require('plugin-name')` is called during `init.lua`
- **THEN** the searcher SHALL return a no-op proxy table

#### Scenario: Plugin loaded via require() is not double-loaded

- **WHEN** a plugin was already loaded via `require()` during `init.lua` and `load_plugins` runs afterward
- **THEN** `load_plugins` SHALL skip re-executing that plugin's `init.lua`

## Operations

### Requirement: Lock file format

The system SHALL use a TOML lock file at `$XDG_CONFIG_HOME/yeet/plugins.lock` (falling back to `~/.config/yeet/plugins.lock`). Each plugin entry SHALL be keyed by its normalized URL path and contain `commit` (string), `sha256` (string), and optionally `branch` (string) and `tag` (string) fields.

#### Scenario: Lock file with one plugin

- **WHEN** a lock file exists with content `[plugins."github.com/user/yeet-nord"]` containing `commit = "abc123"` and `sha256 = "def456"`
- **THEN** the system SHALL parse it as one plugin entry with the given commit and hash

#### Scenario: Lock file does not exist

- **WHEN** no `plugins.lock` file exists in the config directory
- **THEN** the system SHALL treat the plugin set as having no locked versions

### Requirement: Plugin sync restores locked versions

The `:pluginsync` command SHALL read the lock file and, for each registered plugin, check out the exact commit specified. If a plugin is not yet cloned, it SHALL be cloned and checked out to the locked commit. Sync SHALL use the configured concurrency for parallel git operations.

#### Scenario: Sync with existing lock file and missing plugin directory

- **WHEN** the lock file contains an entry for plugin A with commit `abc123` and the plugin directory does not exist
- **THEN** the system SHALL clone the repository and check out commit `abc123`

#### Scenario: Sync with existing lock file and existing plugin directory

- **WHEN** the lock file contains an entry for plugin A with commit `abc123` and the plugin directory exists at a different commit
- **THEN** the system SHALL fetch and check out commit `abc123`

#### Scenario: Sync with no lock file

- **WHEN** no lock file exists and `:pluginsync` is executed
- **THEN** the system SHALL report that no lock file exists and suggest running `:pluginupdate` first

#### Scenario: Sync verifies SHA-256

- **WHEN** a plugin is checked out and the tree SHA-256 does not match the lock file entry
- **THEN** the system SHALL report an integrity error for that plugin

### Requirement: Plugin update resolves latest versions

The `:pluginupdate` command SHALL, for each registered plugin, fetch remote tags/branches and resolve the latest version matching the configured constraints. The resolved commit SHA and tree SHA-256 SHALL be written to the lock file. Update SHALL use the configured concurrency for parallel git operations.

#### Scenario: Update plugin with semver constraint

- **WHEN** plugin A is registered with `version = ">=1.0, <2.0"` and remote tags include `v1.3.0` and `v2.0.0`
- **THEN** the system SHALL resolve `v1.3.0`, check out its commit, and write the commit SHA, tag, and tree SHA-256 to the lock file

#### Scenario: Update plugin with no version constraint

- **WHEN** plugin A is registered with no version constraint and branch `main`
- **THEN** the system SHALL fetch and check out the latest commit on `main` and write the commit SHA to the lock file

#### Scenario: Update plugin with no version constraint and no branch

- **WHEN** plugin A is registered with no version constraint and no branch
- **THEN** the system SHALL fetch and check out the latest commit on the remote HEAD branch and write the commit SHA to the lock file

#### Scenario: Update creates lock file if missing

- **WHEN** no lock file exists and `:pluginupdate` is executed
- **THEN** the system SHALL create the lock file with entries for all registered plugins

#### Scenario: No matching version found

- **WHEN** plugin A is registered with `version = ">=3.0"` and no remote tags match
- **THEN** the system SHALL report an error for that plugin and not update its lock file entry

### Requirement: Plugin storage location

The system SHALL store cloned plugin repositories in `$XDG_DATA_HOME/yeet/plugins/<owner>/<repo>/` (falling back to `~/.local/share/yeet/plugins/`). The `<owner>/<repo>` path SHALL be derived from the git URL.

#### Scenario: GitHub URL storage path

- **WHEN** a plugin is registered with URL `https://github.com/aserowy/yeet-nord`
- **THEN** the plugin SHALL be stored at `$XDG_DATA_HOME/yeet/plugins/aserowy/yeet-nord/`

#### Scenario: XDG_DATA_HOME not set

- **WHEN** `$XDG_DATA_HOME` is not set and the user's home directory is `/home/user`
- **THEN** plugins SHALL be stored under `/home/user/.local/share/yeet/plugins/`

### Requirement: Unregistered plugin cleanup on sync and update

On `:pluginsync` and `:pluginupdate`, the system SHALL compare the registered plugin list against the lock file and data directory. Any plugins present in the lock file or data directory but not in the current registration list SHALL be removed from both the lock file and the data directory.

#### Scenario: Plugin removed from init.lua then sync

- **WHEN** plugin A was previously synced but is no longer registered in `init.lua` and `:pluginsync` is executed
- **THEN** the system SHALL delete plugin A's directory from the data folder and remove its entry from the lock file

#### Scenario: Plugin removed from init.lua then update

- **WHEN** plugin A was previously updated but is no longer registered in `init.lua` and `:pluginupdate` is executed
- **THEN** the system SHALL delete plugin A's directory from the data folder and remove its entry from the lock file

#### Scenario: Orphaned directory with no lock entry

- **WHEN** a plugin directory exists in the data folder but has no lock file entry and is not registered
- **THEN** the system SHALL delete the orphaned directory on sync or update

### Requirement: Dependencies are synced and updated alongside parents

When syncing or updating, the system SHALL process plugin dependencies using the same logic as top-level plugins. Dependencies SHALL appear in the lock file as their own entries.

#### Scenario: Update resolves dependency versions

- **WHEN** plugin A depends on library B with `version = ">=0.5"` and `:pluginupdate` is executed
- **THEN** the system SHALL resolve library B's latest matching version and write it to the lock file

#### Scenario: Sync restores dependency

- **WHEN** the lock file contains an entry for dependency B and `:pluginsync` is executed
- **THEN** the system SHALL clone/checkout dependency B at the locked commit

### Requirement: Shallow clones for tagged versions

The system SHALL use shallow clones (depth 1) when cloning a plugin at a specific tag or commit to minimize download size and time.

#### Scenario: Clone at a specific tag

- **WHEN** a plugin is being cloned for the first time and the resolved version is tag `v1.2.0`
- **THEN** the system SHALL perform a shallow clone at that tag

### Requirement: Git operations do not prompt for credentials

The system SHALL configure git operations to never prompt for credentials on stdin. Authentication failures SHALL produce clean error messages per-plugin without blocking the terminal or breaking application state.

#### Scenario: Private HTTPS repo fails cleanly

- **WHEN** a plugin URL points to a private HTTPS repository and `:pluginupdate` is executed
- **THEN** the system SHALL report an authentication error for that plugin and continue processing other plugins

## Commands

### Requirement: pluginlist command

The system SHALL provide a `:pluginlist` command that synchronously prints the list of registered plugins with their status. For each plugin, the output SHALL include the plugin URL, resolved version/commit, and status (`loaded`, `error`, `missing`). For plugins with `error` or `missing` status, the error message SHALL be included. Each line SHALL be colored by status: loaded in success color, missing in warning color, error in error color.

#### Scenario: All plugins loaded

- **WHEN** the user executes `:pluginlist` and all registered plugins are loaded
- **THEN** the system SHALL print each plugin's URL and status as `loaded`

#### Scenario: Some plugins with errors

- **WHEN** the user executes `:pluginlist` and plugin A failed to load with error "attempt to call nil"
- **THEN** the output SHALL show plugin A with status `error` and the message "attempt to call nil"

#### Scenario: Missing plugins

- **WHEN** the user executes `:pluginlist` and plugin B is missing from disk
- **THEN** the output SHALL show plugin B with status `missing`

#### Scenario: No plugins registered

- **WHEN** the user executes `:pluginlist` and no plugins are registered
- **THEN** the system SHALL print a message indicating no plugins are registered

#### Scenario: Dependencies shown

- **WHEN** the user executes `:pluginlist` and plugin A has a dependency on library B
- **THEN** both plugin A and library B SHALL appear in the output

#### Scenario: Loaded plugin shown in success color

- **WHEN** the user executes `:pluginlist` and plugin A is loaded
- **THEN** plugin A's line SHALL be rendered in the `SuccessFg` theme color

#### Scenario: Missing plugin shown in warning color

- **WHEN** the user executes `:pluginlist` and plugin B is missing from disk
- **THEN** plugin B's line SHALL be rendered in the `WarningFg` theme color

#### Scenario: Error plugin shown in error color

- **WHEN** the user executes `:pluginlist` and plugin C failed to load
- **THEN** plugin C's line SHALL be rendered in the `ErrorFg` theme color

### Requirement: pluginsync command

The system SHALL provide a `:pluginsync` command that dispatches an async task to restore all registered plugins to their locked versions. Progress and results SHALL be shown in the tasks window.

#### Scenario: Sync succeeds

- **WHEN** the user executes `:pluginsync` and all plugins sync successfully
- **THEN** the system SHALL print a success message in the command line with the number of plugins synced

#### Scenario: Sync with missing lock file

- **WHEN** the user executes `:pluginsync` and no lock file exists
- **THEN** the system SHALL print an error in the command line suggesting to run `:pluginupdate` first

#### Scenario: Sync with integrity error

- **WHEN** the user executes `:pluginsync` and a plugin's tree SHA-256 does not match the lock file
- **THEN** the system SHALL print an integrity error for that plugin in the command line

### Requirement: pluginupdate command

The system SHALL provide a `:pluginupdate` command that dispatches an async task to resolve and fetch the latest allowed versions for all registered plugins. The lock file SHALL be updated with new commit SHAs and tree hashes. Progress and results SHALL be shown in the tasks window.

#### Scenario: Update succeeds

- **WHEN** the user executes `:pluginupdate` and all plugins update successfully
- **THEN** the system SHALL print a success message in the command line with the number of plugins updated and the lock file SHALL be written

#### Scenario: Update with version resolution failure

- **WHEN** the user executes `:pluginupdate` and plugin A has no matching remote tag for its version constraint
- **THEN** the system SHALL print an error for plugin A in the command line while other plugins are still updated

#### Scenario: First update creates lock file

- **WHEN** the user executes `:pluginupdate` and no lock file exists
- **THEN** the system SHALL create the lock file with entries for all resolved plugins

### Requirement: Sync and update clean up unregistered plugins

The `:pluginsync` and `:pluginupdate` commands SHALL remove plugins from the lock file and data directory that are no longer registered in `init.lua`.

#### Scenario: Cleanup reported in output

- **WHEN** `:pluginupdate` removes two unregistered plugins
- **THEN** the system SHALL print a message in the command line listing the removed plugins

## Hooks

### Requirement: Plugin identity remains yeet-directory-icons
The integrated plugin SHALL use logical/plugin name `yeet-directory-icons` through the existing plugin configuration/loading flow.

#### Scenario: Runtime references yeet-directory-icons identity
- **WHEN** plugin loading registers the directory icon plugin from user configuration
- **THEN** runtime/plugin identity is `yeet-directory-icons`

### Requirement: Existing plugin loading is used for directory rendering
At startup, existing plugin loading SHALL make `yeet-directory-icons` available to directory buffer rendering so the plugin can mutate bufferlines via hooks.

#### Scenario: Directory rendering invokes plugin mutation hooks through configured plugin
- **WHEN** yeet starts and opens a directory buffer with `yeet-directory-icons` configured and available
- **THEN** mutation hook calls fire for each bufferline and are served by `yeet-directory-icons`

#### Scenario: Plugin load sets icon-column width
- **WHEN** `yeet-directory-icons` executes its `on_window_create` hook
- **THEN** shared `@yeet-buffer` icon-column width is configured to `1`

#### Scenario: Plugin unavailable keeps zero-width icon column
- **WHEN** `yeet-directory-icons` is unavailable or not configured
- **THEN** shared `@yeet-buffer` icon-column width remains at the default `0` and no per-bufferline hooks are invoked

#### Scenario: Plugin configuration/load failure is reported
- **WHEN** `yeet-directory-icons` is configured but fails to load
- **THEN** the system reports a plugin loading diagnostic and continues with icon-column width `0`

### Requirement: Mutation hook fires for all buffer types with buffer metadata object
The core SHALL invoke the `on_bufferline_mutate` hook for all buffer types when bufferlines are created or updated. Each hook invocation SHALL provide buffer metadata as a read-only `buffer` object (`ctx.buffer`) containing `type` (e.g., `"directory"`, `"content"`, `"help"`, `"quickfix"`, `"tasks"`) and optionally `path` (parent dir for directory, file path for content; absent/nil for help, quickfix, tasks). The plugin decides which buffer types to process by checking `ctx.buffer.type`. The `buffer_type` parameter in the Rust API SHALL use a `BufferType` enum instead of `&str`.

#### Scenario: Hook fires for directory buffer entries
- **WHEN** the core handles `EnumerationChanged`, `EnumerationFinished`, or `PathsAdded` and processes a bufferline
- **THEN** the hook fires with `ctx.buffer.type` set to `"directory"` and `ctx.buffer.path` set to the parent directory path

#### Scenario: Hook fires for content buffer entries
- **WHEN** the core populates a content buffer (file preview)
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"content"` and `ctx.buffer.path` set to the file path

#### Scenario: Hook fires for help buffer entries
- **WHEN** the core populates a help buffer
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"help"` and `ctx.buffer.path` absent (nil)

#### Scenario: Hook fires for quickfix buffer entries
- **WHEN** the core populates a quickfix buffer
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"quickfix"` and `ctx.buffer.path` absent (nil)

#### Scenario: Hook fires for tasks buffer entries
- **WHEN** the core populates a tasks buffer
- **THEN** the hook fires for each bufferline with `ctx.buffer.type` set to `"tasks"` and `ctx.buffer.path` absent (nil)

#### Scenario: Plugin directly mutates bufferline via hook context
- **WHEN** the plugin receives a hook call with bufferline and `ctx.buffer` metadata object
- **THEN** the plugin can mutate `prefix`, `content`, `search_char_position`, `signs`, and `icon` fields in-place on the context table; the `buffer` metadata object is read-only

### Requirement: Deferred PathsAdded hooks fire on flush
When `PathsAdded` events are deferred during Insert mode, the per-bufferline mutation hooks SHALL also be deferred. Hooks fire when deferred events are flushed (after leaving Insert mode).

#### Scenario: Deferred PathsAdded hooks fire after leaving Insert mode
- **WHEN** `PathsAdded` events are queued during Insert mode and the user leaves Insert mode
- **THEN** the deferred events are flushed and mutation hooks fire for each new bufferline at flush time

### Requirement: Plugin-manager workflows are unchanged
The system SHALL NOT require changes to plugin-manager commands/workflows (install/update/sync/lock) for this feature.

#### Scenario: Feature uses normal plugin configuration path
- **WHEN** a user installs/configures `yeet-directory-icons` through their normal setup
- **THEN** directory icon integration works without introducing new plugin-manager behavior

### Requirement: Plugins can provide help pages
Plugins SHALL be able to provide help documentation by placing markdown files in a `docs/help/` directory within their plugin directory. The `:help` command SHALL discover these files at runtime and include them as searchable help pages.

#### Scenario: Plugin help page is discoverable
- **WHEN** a plugin has a `docs/help/directory-icons.md` file
- **THEN** `:help directory-icons` shows the plugin's help content

#### Scenario: Core help takes priority
- **WHEN** a topic matches both a core help page and a plugin help page
- **THEN** the core help page is shown

#### Scenario: Plugin not loaded means no help
- **WHEN** a plugin is not loaded/configured
- **THEN** its help pages are not available in `:help`

### Requirement: Plugin-specific documentation lives in plugin repos
Plugin-specific documentation (token references, usage guides, configuration) SHALL be maintained in each plugin's own `docs/help/` directory, not in core `docs/help/` files. Core documentation SHALL only document core concepts and SHALL NOT reference optional plugin-specific tokens or behavior.

#### Scenario: Plugin token docs in plugin repo
- **WHEN** a user wants to learn about `DirectoryIconsColor*` tokens
- **THEN** they find the documentation in the `yeet-directory-icons` plugin's help page, not in core `docs/help/theme.md`
