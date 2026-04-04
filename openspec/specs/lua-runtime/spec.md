### Requirement: Lua runtime initialization
The system SHALL embed a Lua 5.4 runtime using the mlua crate at application startup, before any UI rendering occurs.

#### Scenario: Successful initialization without config file
- **WHEN** yeet starts and no `init.lua` exists at the config path
- **THEN** the Lua runtime is initialized but no user script is executed, and yeet starts with default settings

#### Scenario: Successful initialization with config file
- **WHEN** yeet starts and `init.lua` exists at `$XDG_CONFIG_HOME/yeet/init.lua`
- **THEN** the Lua runtime loads and executes the file before UI rendering begins

### Requirement: Config file location follows XDG
The system SHALL look for `init.lua` at `$XDG_CONFIG_HOME/yeet/init.lua`. If `$XDG_CONFIG_HOME` is not set, the system SHALL fall back to `~/.config/yeet/init.lua`.

#### Scenario: XDG_CONFIG_HOME is set
- **WHEN** `$XDG_CONFIG_HOME` is set to `/custom/config`
- **THEN** the system loads `/custom/config/yeet/init.lua`

#### Scenario: XDG_CONFIG_HOME is not set
- **WHEN** `$XDG_CONFIG_HOME` is not set and the user's home directory is `/home/user`
- **THEN** the system loads `/home/user/.config/yeet/init.lua`

### Requirement: Global y table is exposed to Lua
The system SHALL expose a global table named `y` to the Lua environment. This table serves as the namespace for all yeet configuration APIs.

#### Scenario: y table is accessible in init.lua
- **WHEN** `init.lua` contains `y.theme.StatusLineFg = '#ffffff'`
- **THEN** the assignment executes without error and the value is accessible from the Rust side

### Requirement: Lua errors are reported gracefully
The system SHALL NOT crash if `init.lua` contains syntax errors or runtime errors. The system SHALL log the error and continue startup with default settings.

#### Scenario: Syntax error in init.lua
- **WHEN** `init.lua` contains invalid Lua syntax
- **THEN** yeet logs an error message indicating the file and error, and starts with default theme colors

#### Scenario: Runtime error in init.lua
- **WHEN** `init.lua` raises a runtime error (e.g., calling a nil value)
- **THEN** yeet logs the error with a stack trace and starts with default theme colors
