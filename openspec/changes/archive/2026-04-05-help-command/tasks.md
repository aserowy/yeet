## 1. Model: Buffer and Window types

- [x] 1.1 Add `HelpBuffer { buffer: TextBuffer }` struct to the `Buffer` enum in `yeet-frontend/src/model/mod.rs`
- [x] 1.2 Add `Window::Help(ViewPort)` variant to the `Window` enum in `yeet-frontend/src/model/mod.rs`
- [x] 1.3 Handle `Window::Help` in all existing match arms on `Window` (focused_viewport, focused_window_mut, close_focused, etc.)

## 2. Read-only mode blocking

- [x] 2.1 Add `Buffer::Help(_)` to the mode-change blocking match in `yeet-frontend/src/update/mode.rs` alongside `Buffer::Tasks(_)` and `Buffer::QuickFix(_)`
- [x] 2.2 Write tests verifying that Insert and Normal mode transitions are blocked on Help buffers

## 3. Help content embedding

- [x] 3.1 Create `docs/help/` directory with an index markdown file (help landing page)
- [x] 3.2 Create initial topic help pages using the structured entry format: `#` page title, `##` sections, `` ### `identifier` `` entries (e.g., commands, keybindings, navigation)
- [x] 3.3 Embed help markdown files via `include_str!` and create a topic-to-entry lookup function that maps topic strings to the matching page and entry line offset

## 4. Help command dispatch

- [x] 4.1 Add `:help` command entry to `command::execute()` in `yeet-frontend/src/update/command/mod.rs`
- [x] 4.2 Implement `help::open(app, topic)` function that resolves the topic, creates a `HelpBuffer` with syntax-highlighted content, and opens it in a horizontal split with focus on the help pane (bottom)
- [x] 4.3 Handle bare `:help` (no args) by opening the index page
- [x] 4.4 Handle `:help <topic>` by resolving the topic to a page and entry line offset, opening the matching page scrolled to that entry, or returning an error for unknown topics

## 5. Syntax highlighting

- [x] 5.1 Reuse `syntax::highlight` from `yeet-frontend/src/task/syntax.rs` to produce ANSI-escaped lines from the embedded markdown content when building the help buffer

## 6. View rendering

- [x] 6.1 Add `Window::Help(vp)` match arm to `render_window` in `yeet-frontend/src/view/buffer.rs`, rendering via `render_buffer_slot` (same as QuickFix/Tasks)
- [x] 6.2 Add `Buffer::Help(buf)` match arm to `render_buffer_slot` calling `buffer_view` with `buf.buffer`

## 7. Window close support

- [x] 7.1 Ensure `Window::Help` is handled by `close_focused()` so `:q` removes the help pane and restores focus

## 8. Tests

- [x] 8.1 Write tests for topic resolution (known topic returns page + line offset, unknown topic returns error)
- [x] 8.2 Write tests for help buffer open (horizontal split created, help pane focused, buffer is Help variant)
- [x] 8.3 Write tests for help buffer close (split removed, focus restored)
- [x] 8.4 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
