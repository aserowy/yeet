## Context

`sync()` and `update()` in `yeet-plugin` iterate over plugins sequentially. The task handlers in `yeet-frontend` receive a `concurrency` value from `Task::PluginSync(specs, concurrency)` / `Task::PluginUpdate(specs, concurrency)` but bind it as `_concurrency`.

## Goals / Non-Goals

**Goals:**

- Git operations (clone, fetch, checkout) run in parallel up to the configured concurrency limit
- Sequential parts (lock file read/write, cleanup) remain sequential

**Non-Goals:**

- Async rewrite of yeet-plugin (keep blocking gix calls)

## Decisions

### 1. tokio::task::spawn_blocking with Semaphore

Since gix operations are blocking, wrap each plugin's git work in `tokio::task::spawn_blocking`. Use a `tokio::sync::Semaphore` with `concurrency` permits to limit how many blocking tasks run in parallel. This stays consistent with the rest of the codebase which uses tokio throughout.

The `sync()` and `update()` functions become `async` since they use `tokio::task::spawn_blocking` and semaphore `.acquire().await`.

### 2. Signature change

```rust
pub fn sync(specs, lock_file_path, data_path, concurrency: usize) -> Result<SyncResult, SyncError>
pub fn update(specs, lock_file_path, data_path, concurrency: usize) -> Result<UpdateResult, UpdateError>
```

### 3. Task handlers pass concurrency through

Change `_concurrency` to `concurrency` in both match arms and pass to `sync()`/`update()`.
