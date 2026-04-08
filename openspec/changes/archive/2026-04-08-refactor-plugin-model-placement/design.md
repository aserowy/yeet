## Context

`Model` currently holds `plugin_states` and `plugin_concurrency` as top-level fields. `State` holds all other runtime state (history, marks, qfix, etc.). `Settings` holds all configuration (theme, window settings, startup path, etc.). The plugin fields are misplaced.

The `run()` function takes 4 parameters: `settings`, `lua`, `plugin_states`, `plugin_concurrency`. The last two should be absorbed into existing structs.

## Goals / Non-Goals

**Goals:**

- `plugin_states` moves to `State` alongside other runtime data
- `plugin_concurrency` moves to `Settings` alongside other configuration
- `run()` signature simplifies to `(settings, lua, plugin_states)` — concurrency is already in settings, plugin_states initializes state
- Lua-configurable settings follow a consistent pattern extensible to future settings

**Non-Goals:**

- Changing plugin behavior
- Adding new Lua-configurable settings (just establishing the pattern)

## Decisions

### 1. plugin_concurrency in Settings with default

**Decision**: Add `plugin_concurrency: usize` to `Settings` with default value 4. Set it from Lua during initialization in `yeet/src/lua.rs` before passing `Settings` to `run()`.

**Rationale**: Follows the existing pattern where `Settings` is populated before `run()` is called. The default in `Settings::default()` means it works without Lua config.

### 2. plugin_states in State

**Decision**: Add `plugin_states: Vec<PluginState>` to `State`. Initialize it via `run()` which receives plugin_states and sets `model.state.plugin_states` during Model construction.

**Rationale**: `State` holds all mutable runtime data. Plugin states are runtime data — they're populated during startup and read by `:pluginlist`.

### 3. run() signature change

**Decision**: Change `run(settings, lua, plugin_states, plugin_concurrency)` to `run(settings, lua, plugin_states)`. Concurrency is already in `settings` at this point.

**Rationale**: Reduces parameter count. Settings carries all config, plugin_states is the only remaining runtime init data that can't go in Settings (it's not config, it's state).

### 4. Command dispatch reads from model.state and model.settings

**Decision**: `command::execute` drops the `plugin_states` and `plugin_concurrency` parameters. Instead it receives `&State` (already does) and `&Settings` (already does via `theme`). Access via `state.plugin_states` and `settings.plugin_concurrency`.

Currently execute takes `theme: &Theme` — this should change to `settings: &Settings` so it has access to all settings including `plugin_concurrency`. Theme is accessed as `settings.theme`.

**Rationale**: Avoids threading individual fields through function signatures. The command dispatch already has access to state and settings through the model decomposition.
