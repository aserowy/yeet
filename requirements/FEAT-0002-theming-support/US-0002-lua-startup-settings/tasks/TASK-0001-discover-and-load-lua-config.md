# Task: Discover and Load Lua Config on Startup

## Metadata

- ID: TASK-0001
- Status: done
- Userstory: US-0002

## Motivation

The application needs a deterministic startup path to locate and execute a Lua configuration file so user-defined theme settings can be applied without manual path specification.

## Relevant Acceptance Criteria

- Given a Lua configuration is available through the application's config discovery
- When the application starts
- Then it executes the Lua config and applies the defined theming settings

## Requirements

- Define a single config discovery strategy for Lua config (directory + filename) and document it in code comments: `$XDG_CONFIG_HOME/yeet/init.lua` with fallback to `~/.config/yeet/init.lua`.
- Implement Lua config file discovery during startup before the first render.
- If no Lua config is found, continue startup with default settings (no errors).
- If a Lua config is found but cannot be loaded/executed, surface a clear error via existing logging/print channels and continue with defaults.
- Provide an internal helper that returns the discovered Lua config path (or None) to enable testability.

## Exclusions

- Do NOT add CLI flags or command-line overrides for Lua config paths.
- Do NOT implement the Lua API for theme palette settings in this task.
- Do NOT add hot-reload or runtime reconfiguration; only startup behavior.

## Context

- @yeet/src/main.rs - startup settings construction and initial application launch.
- @yeet-frontend/src/lib.rs - startup flow before initial rendering.
- @yeet-frontend/src/settings.rs - Settings defaults and structure.
- @AGENTS.md - build/test/lint commands.

## Implementation Plan

### Step 1: Define discovery strategy and helper

Add a helper function in the startup path (likely in `yeet/src/main.rs` or a small module) that returns `Option<PathBuf>` for a discovered Lua config using a deterministic search path. Example strategy:

```rust
fn discover_lua_config() -> Option<PathBuf> {
    // Example: $XDG_CONFIG_HOME/yeet/init.lua, fallback to ~/.config/yeet/init.lua
}
```

Document the chosen strategy with a short comment so tests can assert the behavior.

### Step 2: Execute Lua config on startup

Wire discovery into startup so the Lua config is executed before rendering begins. This likely means resolving the config path and passing it into the frontend settings or init flow.

### Step 3: Handle error and no-config cases

Ensure the no-config path is silent (no error). Ensure error cases are logged or emitted via existing `PrintContent::Error` patterns without crashing the app.

When surfacing errors, include context indicating that the Lua config is expected to define settings under the top-level `y` table (e.g., `y.theme`).

## Examples

- If `~/.config/yeet/init.lua` exists, it is discovered and executed on startup.
- If no config exists, startup completes using default theme palette values.
