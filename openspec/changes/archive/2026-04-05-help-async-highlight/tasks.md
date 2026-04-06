## 1. Message and Task variants

- [x] 1.1 Add `Message::HelpHighlighted(usize, Vec<String>)` variant to `yeet-frontend/src/event.rs` where the `usize` is the `buffer_id`
- [x] 1.2 Add `Task::HighlightHelp(usize, String)` variant to `yeet-frontend/src/task/mod.rs` where the `usize` is the `buffer_id` and the `String` is the raw markdown content
- [x] 1.3 Implement the `Task::HighlightHelp` handler in `run_task` in `yeet-frontend/src/task/mod.rs`: acquire the shared highlighter lock, highlight the content as markdown using `syntax::highlight`-style logic (find md syntax, iterate `LinesWithEndings`, produce ANSI-escaped strings), and send `Message::HelpHighlighted(buffer_id, lines)` via the sender

## 2. Open help with raw content and spawn highlight task

- [x] 2.1 In `help::open()` in `yeet-frontend/src/update/command/help.rs`, remove the `highlight_markdown` function and its syntect imports
- [x] 2.2 Replace the highlighted lines construction with raw content lines: split `topic_match.content` lines and create `BufferLine`s from the raw strings
- [x] 2.3 After creating the help buffer and setting cursor/viewport, return `Action::Task(Task::HighlightHelp(buffer_id, topic_match.content.to_string()))` in the returned actions vec, where `buffer_id` is the id assigned to the new help buffer

## 3. Handle highlighted content message

- [x] 3.1 Route `Message::HelpHighlighted` in `update_with_message` in `yeet-frontend/src/update/mod.rs` to a new handler function
- [x] 3.2 Implement the handler: look up the buffer directly in `contents.buffers` by the `buffer_id` from the message. If it exists and is `Buffer::Help`, split the highlighted strings via `split_terminator('\n')` into `BufferLine`s and replace the `TextBuffer` lines. If the buffer no longer exists, silently return.

## 4. Tests

- [x] 4.1 Write a test verifying that `help::open` returns an `Action::Task(Task::HighlightHelp(_, _))` action containing the buffer_id and raw content
- [x] 4.2 Write a test verifying that the help buffer opened by `help::open` contains raw (unhighlighted) content lines
- [x] 4.3 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
