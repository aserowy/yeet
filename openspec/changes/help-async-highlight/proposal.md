## Why

Opening the help buffer blocks the main thread because `highlight_markdown` calls `SyntaxSet::load_defaults_newlines()` and runs syntect highlighting synchronously in `help::open()`. This causes a visible lag every time `:help` is executed. The preview system already solves this problem — it highlights file content asynchronously via `Task::LoadPreview` and delivers results through `Message::PreviewLoaded`. The help buffer should follow the same pattern.

## What Changes

- Remove synchronous `highlight_markdown` call from `help::open()`. Instead, open the help buffer immediately with unhighlighted content and return an `Action` that spawns a highlighting task.
- Add a new `Task` variant for highlighting help content (embedded strings, not file paths).
- Add a new `Message` variant to deliver highlighted help content back to the main thread.
- On receiving the highlighted content message, update the existing help buffer in-place, preserving cursor and viewport position.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `help`: Help buffer syntax highlighting SHALL be performed asynchronously via the task system, following the same pattern as file preview highlighting.

## Impact

- `yeet-frontend/src/update/command/help.rs`: Remove `highlight_markdown`, open buffer with raw content, return action to spawn highlight task.
- `yeet-frontend/src/task/mod.rs`: Add `Task::HighlightHelp` variant and handler using shared syntect highlighter.
- `yeet-frontend/src/event.rs`: Add `Message::HelpHighlighted` variant.
- `yeet-frontend/src/update/mod.rs`: Route new message to help buffer update handler.
- `yeet-frontend/src/action.rs`: Add action variant or use existing `Action::EmitMessages` pattern.
