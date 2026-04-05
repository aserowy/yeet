## Context

Currently `help::open()` calls `highlight_markdown()` synchronously, which loads `SyntaxSet::load_defaults_newlines()` and `ThemeSet::load_defaults()` on every invocation. This blocks the main thread visibly.

The existing preview system handles the same problem: `Task::LoadPreview` runs `syntax::highlight()` asynchronously on a tokio-spawned task, and `Message::PreviewLoaded(Preview)` delivers the result back. The task runner already holds a shared `Arc<Mutex<(SyntaxSet, ThemeSet)>>` highlighter instance, so no additional loading is needed.

## Goals / Non-Goals

**Goals:**

- Move help content syntax highlighting off the main thread using the existing task system.
- Show the help buffer immediately with unhighlighted (raw) content, then update it when highlighting completes.
- Reuse the shared highlighter from the task runner (no extra `SyntaxSet` load).
- Preserve cursor and viewport position when the highlighted content replaces the raw content.

**Non-Goals:**

- Caching highlighted help content across `:help` invocations.
- Changing the highlighting algorithm itself.
- Refactoring the preview system.

## Decisions

**Open with raw content, then replace asynchronously**

`help::open()` will create the `HelpBuffer` with raw (unhighlighted) lines from the embedded markdown, create the split, set cursor/viewport, and return an `Action::Task(Task::HighlightHelp(...))`. The task runner highlights the content using the shared highlighter and sends `Message::HelpHighlighted(Vec<String>)` back. A new handler finds the `Window::Help` viewport in the current tab, retrieves the `buffer_id`, and replaces the buffer lines in-place while preserving cursor and viewport position.

Alternative considered: Showing a loading placeholder instead of raw content. Rejected because the raw markdown is already readable, and showing it immediately gives instant feedback while highlighting catches up (typically < 100ms on a background thread).

**New Task variant `Task::HighlightHelp(usize, String)` carrying buffer_id and raw content**

The task carries the `buffer_id` assigned at open time so the message handler knows exactly which help buffer to update. Multiple help buffers can be open simultaneously (in different splits or tabs), and each gets its own highlighting task keyed by its unique `buffer_id`.

The help content is embedded `&'static str` but the task system needs owned data for the `Send` boundary. The content is small (a few KB), so cloning to `String` is fine.

**New Message variant `Message::HelpHighlighted(usize, Vec<String>)`**

Follows the same pattern as `Message::PreviewLoaded(Preview::Content(path, Vec<String>))`. The `usize` is the `buffer_id` identifying which help buffer to update. The `Vec<String>` contains ANSI-escaped highlighted lines.

**Look up the help buffer by buffer_id directly**

On receiving `Message::HelpHighlighted(buffer_id, lines)`, look up the buffer directly in `contents.buffers` by `buffer_id`. If it exists and is a `Buffer::Help`, replace its `TextBuffer` lines. If the buffer no longer exists (user closed it), silently drop the message. No window tree traversal needed — the `buffer_id` is the direct key.

## Risks / Trade-offs

- [Help buffer replaced after user scrolled] → Line count stays identical (same source content, just highlighted). Cursor and viewport indices remain valid. The visual content updates in place without jarring scrolling.
- [User closes help before highlighting finishes] → The message handler must check that a `Window::Help` and its buffer still exist before updating. If not found, the message is silently dropped.
- [Task system already holds highlighter lock] → The help content is small, so the lock is held briefly. Preview tasks are unlikely to contend meaningfully.
