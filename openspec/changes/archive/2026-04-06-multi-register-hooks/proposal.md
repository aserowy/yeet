## Why

Currently each hook (`y.hook.on_window_create`) accepts only a single function. Assigning a second function overwrites the first. This prevents modular configuration where separate concerns (e.g., one hook for line numbers, another for wrap settings) are defined independently. Supporting multiple registrations per hook enables composable configuration.

## What Changes

- Replace the direct function assignment (`y.hook.on_window_create = function(ctx) end`) with a registration API (`y.hook.on_window_create:add(function(ctx) end)`)
- Each hook name holds an ordered list of callbacks instead of a single function
- When a hook fires, all registered callbacks are invoked in registration order, each receiving the same context table — mutations from earlier callbacks are visible to later ones
- **BREAKING**: `y.hook.on_window_create = function(ctx) end` no longer works; users must use `y.hook.on_window_create:add(function(ctx) end)`

## Capabilities

### New Capabilities

### Modified Capabilities

- `lua-callbacks`: The hook registration model changes from single-function assignment to multi-registration via `:add()`. Invocation iterates all registered callbacks in order.

## Impact

- **yeet-lua crate** (`lib.rs`): Hook table initialization changes — each hook name gets a userdata or table with an `:add()` method and internal list
- **yeet-lua crate** (`hook.rs`): Invocation logic iterates the callback list instead of calling a single function
- **docs/help/hooks.md**: User-facing API examples updated from assignment to `:add()`
- **User configuration** (`init.lua`): **BREAKING** — existing `y.hook.on_window_create = fn` assignments must migrate to `y.hook.on_window_create:add(fn)`
