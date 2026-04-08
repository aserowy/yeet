## Context

`Message::Error(String)` is used throughout the codebase as a general-purpose user-facing message channel from tasks and commands. The commandline renderer already supports theme-aware `PrintContent` variants (`Error`, `Warning`, `Information`). The gap is between `Message` (which only has `Error`) and `PrintContent` (which has multiple severities).

## Goals / Non-Goals

**Goals:**

- `Message::Log(LogMessage)` replaces `Message::Error(String)`
- `LogMessage` has a severity field and convenience constructors
- Handler maps severity to `PrintContent` variant using existing theme tokens
- Plugin sync/update tasks use correct severity per message type

**Non-Goals:**

- Changing the `PrintContent` enum (already has the variants we need)
- Adding log history or persistence

## Decisions

### 1. LogSeverity enum

```rust
pub enum LogSeverity {
    Error,
    Warning,
    Information,
}
```

Placed in `event.rs` alongside the `Message` enum.

### 2. Message::Log(LogSeverity, String)

Replace `Message::Error(String)` with `Message::Log(LogSeverity, String)`. No wrapper struct — the tuple variant keeps it simple.

### 3. Message handler maps severity to PrintContent

In `update/mod.rs`, the `Message::Log` handler maps:
- `LogSeverity::Error` → `PrintContent::Error` (rendered with `ErrorFg`)
- `LogSeverity::Warning` → `PrintContent::Warning` (rendered with `WarningFg`)
- `LogSeverity::Information` → `PrintContent::Information` (rendered with `InformationFg`)

### 4. Plugin task severity mapping

Sync and update tasks:
- `result.errors` entries → `Message::Log(LogSeverity::Error, ...)`
- `result.removed` entries → `Message::Log(LogSeverity::Warning, ...)`
- Success summary → `Message::Log(LogSeverity::Information, ...)`
- Path resolution failure → `Message::Log(LogSeverity::Error, ...)`

### 5. Migration of existing Message::Error sites

All existing `Message::Error(msg)` → `Message::Log(LogSeverity::Error, msg)`. This preserves current behavior.
