## Why

The `plugin_concurrency` setting is passed through `Task::PluginSync` and `Task::PluginUpdate` but discarded with `_concurrency` in the task handlers. The `sync()` and `update()` functions in `yeet-plugin` process plugins sequentially. This means all network I/O (clone, fetch) happens one plugin at a time regardless of the configured concurrency.

## What Changes

- Add `concurrency: usize` parameter to `sync()` and `update()` in `yeet-plugin`
- Process plugins in parallel using `std::thread::scope` with a channel-based semaphore to limit concurrent git operations
- Thread the concurrency value from task handlers instead of discarding it

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

_None — this implements existing specified behavior that was not yet enforced_

## Impact

- **yeet-plugin/sync.rs**: `sync()` gains `concurrency` param, uses parallel execution
- **yeet-plugin/update.rs**: `update()` gains `concurrency` param, uses parallel execution
- **yeet-frontend/task/mod.rs**: Task handlers pass `concurrency` instead of discarding it
