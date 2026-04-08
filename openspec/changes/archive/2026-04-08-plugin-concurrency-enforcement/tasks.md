## 1. Make sync and update async with concurrency

- [x] 1.1 Add `concurrency: usize` parameter to `sync()` in `sync.rs`, make it `async`
- [x] 1.2 Replace sequential for-loop in `sync()` with `tokio::task::spawn_blocking` per plugin, gated by `tokio::sync::Semaphore(concurrency)`, collect results after all handles join
- [x] 1.3 Add `concurrency: usize` parameter to `update()` in `update.rs`, make it `async`
- [x] 1.4 Refactor `update_single_plugin` to return the `LockEntry` instead of mutating `&mut LockFile` directly, so it can run in `spawn_blocking`
- [x] 1.5 Replace sequential for-loop in `update()` with `tokio::task::spawn_blocking` per plugin gated by semaphore, collect results and apply lock entries after all handles join

## 2. Thread concurrency from task handlers

- [x] 2.1 Change `_concurrency` to `concurrency` in `Task::PluginSync` match arm, pass to `sync()`, and `.await` the result
- [x] 2.2 Change `_concurrency` to `concurrency` in `Task::PluginUpdate` match arm, pass to `update()`, and `.await` the result

## 3. Build Verification

- [x] 3.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 3.2 Run `cargo test` and ensure all tests pass
- [x] 3.3 Run `nix build .` and ensure build succeeds
