## Context

Yeet is a Neovim-like TUI file manager with modal editing, a recursive window tree for splits, and several specialized buffer types (Directory, Content, QuickFix, Tasks). The QuickFix and Tasks windows already demonstrate a pattern for read-only, single-viewport buffers opened via horizontal splits with mode-restricted navigation. There is no existing help system or asset embedding infrastructure.

## Goals / Non-Goals

**Goals:**
- Add `:help` and `:help <topic>` commands that open help content in a horizontal split
- Help buffer is read-only, navigable, and closable with `:q`
- Help pages are markdown files embedded at compile time via `include_str!`
- Follow existing patterns (QuickFix/Tasks) for buffer type, window variant, mode blocking, and rendering

**Non-Goals:**
- Hyperlinks or cross-references between help pages
- Search within help pages (beyond standard `/` if already supported)
- Tag-based jumping (like Neovim's `Ctrl-]` on tags)
- Help page authoring tools or generation scripts

## Decisions

### 1. New `Buffer::Help` variant and `Window::Help` variant

Follow the QuickFix/Tasks pattern: add `HelpBuffer { buffer: TextBuffer }` to the `Buffer` enum and `Window::Help(ViewPort)` to the `Window` enum.

**Why not reuse `ContentBuffer`?** ContentBuffer is designed for editable file content with a file path association. A dedicated Help variant allows mode blocking in `mode.rs` without special-casing ContentBuffer, keeps the read-only invariant enforced at the type level, and avoids triggering save/unsaved-changes logic.

### 2. Horizontal split with focus on help pane

Use the same split mechanics as `copen`/`topen`: wrap the focused window leaf in `Window::Horizontal { first: old_window, second: help_window, focus: SplitFocus::Second }` so the help pane appears at the bottom and receives focus.

**Why bottom?** Keeps the user's current context visible at the top while help appears below, similar to how QuickFix and Tasks windows open.

### 3. Read-only via mode blocking

Add `Buffer::Help(_)` to the existing match in `mode.rs` alongside `Buffer::Tasks(_)` and `Buffer::QuickFix(_)`. This blocks transitions to Insert and Normal mode while allowing Navigation and Command mode.

**Why not a separate read-only flag?** The mode blocking pattern is already established and tested for QuickFix/Tasks. Adding a generic read-only flag would be overengineering for this use case.

### 4. Embed help pages with `include_str!`

Use `include_str!("../../docs/help/<filename>.md")` to embed markdown files at compile time. Define a registry mapping topic names to embedded content strings.

**Why `include_str!` over `rust-embed` crate?** No external dependency needed. The help page count is small and known at compile time. `include_str!` is zero-cost and idiomatic for static content. A crate like `rust-embed` would be warranted if we had many files or needed runtime directory scanning.

### 5. Structured help page format

Help pages use a three-level markdown structure:

```markdown
# Page Title

## Section Name

### `identifier`
Description of the entry.

### `another-identifier`
Description of another entry.
```

- `#` — page title (one per file)
- `##` — section grouping (e.g., "Movement", "Commands")
- `` ### `identifier` `` — individual entry with a backtick-wrapped identifier

The `` ### `...` `` pattern is the navigable unit. Future motions (jump to next/previous entry) can search for this pattern. Entry identifiers are unique within a page and serve as the target for `:help <topic>` resolution.

**Why backtick-wrapped identifiers?** Backticks are visually distinct in both raw and highlighted markdown. They create a parseable, unambiguous pattern (`### \`...\``) that won't collide with prose headings. The identifier doubles as the topic lookup key.

### 6. Topic resolution via a static lookup map

Maintain a function or static map that maps topic strings to the corresponding embedded help page and entry line offset. The bare `:help` command maps to an index page. `:help <topic>` finds the page containing a matching `` ### `<topic>` `` entry and scrolls to it. Unknown topics produce an error message.

**Why a static map over filesystem-based lookup?** Content is embedded at compile time, so there is no filesystem to scan. A static map is simple, fast, and exhaustive — topic existence can be checked at compile time.

### 7. Help content stored as markdown source files in the repository

Help pages live in a `docs/help/` directory in the repository as `.md` files. They are authored and maintained manually alongside feature changes. The embedded markdown is syntax highlighted at display time using the existing `syntect` infrastructure in `yeet-frontend/src/task/syntax.rs`.

**Why markdown?** Readable as plain text, familiar format for documentation, and syntect already supports markdown highlighting out of the box. Reusing the existing `syntax::highlight` path (which resolves syntax by extension/content and produces ANSI-escaped lines) means no new rendering dependencies.

## Risks / Trade-offs

**[Manual help page maintenance]** Help pages can drift out of sync with actual commands/keybindings if not updated alongside code changes. → Mitigated by the spec requirement that help pages SHALL be updated when functionality changes. Could add CI checks later to verify coverage.

**[Binary size increase]** Embedded markdown adds to binary size. → Negligible for text content. Even extensive help documentation would add single-digit kilobytes.
