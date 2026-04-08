## Why

`Message::Error(String)` is currently the only mechanism for task results to communicate status to the user. Both sync and update tasks use `Message::Error` for success messages, removal notices, and actual errors — all rendered the same way (red). This makes it hard to distinguish severity at a glance. A `Message::Log` variant with severity levels enables color-coded output matching the existing theme tokens.

## What Changes

- Rename `Message::Error(String)` to `Message::Log(LogMessage)` where `LogMessage` contains a severity enum and a message string
- Add `LogSeverity` enum: `Error`, `Warning`, `Information`
- Update the message handler to render log messages using the corresponding theme token (`ErrorFg`, `WarningFg`, `InformationFg`)
- Rework plugin sync/update task handlers to use appropriate severity: `Error` for `result.errors`, `Warning` for removed plugins, `Information` for success
- Update all existing `Message::Error(...)` call sites to `Message::Log(LogMessage::error(...))`

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

_None — this is an internal refactor with improved rendering but no spec-level behavior changes_

## Impact

- **yeet-frontend/event.rs**: `Message` enum changes (`Error` → `Log`)
- **yeet-frontend/update/mod.rs**: Message handler renders log severity with theme colors
- **yeet-frontend/task/mod.rs**: Plugin sync/update use severity-appropriate log messages
- **All files using Message::Error**: Migrated to `Message::Log` (34 occurrences across 8 files)
