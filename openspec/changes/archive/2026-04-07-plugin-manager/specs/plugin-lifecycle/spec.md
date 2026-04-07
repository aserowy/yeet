## ADDED Requirements

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
