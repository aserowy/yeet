## 1. LogSeverity Type

- [x] 1.1 Add `LogSeverity` enum (`Error`, `Warning`, `Information`) to `event.rs`

## 2. Replace Message::Error with Message::Log

- [x] 2.1 Replace `Message::Error(String)` variant with `Message::Log(LogSeverity, String)` in `event.rs`
- [x] 2.2 Update `Debug` impl for `Message::Log`
- [x] 2.3 Update handler in `update/mod.rs` to match `Message::Log` and map severity to `PrintContent` variant

## 3. Migrate Existing Call Sites

- [x] 3.1 Migrate all `Message::Error(...)` in `task/mod.rs` to `Message::Log(LogSeverity::Error, ...)`
- [x] 3.2 Migrate all `Message::Error(...)` in `update/command/mod.rs`
- [x] 3.3 Migrate all `Message::Error(...)` in `update/command/file.rs`
- [x] 3.4 Migrate all `Message::Error(...)` in `update/command/help.rs`
- [x] 3.5 Migrate all `Message::Error(...)` in `update/command/settings.rs`
- [x] 3.6 Migrate all `Message::Error(...)` in `update/selection.rs`
- [x] 3.7 Migrate all `Message::Error(...)` in `lib.rs` (none found — already clean)

## 4. Rework Plugin Task Severity

- [x] 4.1 In plugin sync task: use `LogMessage::error` for `result.errors`, `LogMessage::warning` for `result.removed`, `LogMessage::information` for success
- [x] 4.2 In plugin update task: use `LogMessage::error` for `result.errors`, `LogMessage::warning` for `result.removed`, `LogMessage::information` for success

## 5. Build Verification

- [x] 5.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 5.2 Run `cargo test` and ensure all tests pass
- [x] 5.3 Run `nix build .` and ensure build succeeds
