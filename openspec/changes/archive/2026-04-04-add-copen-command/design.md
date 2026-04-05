## Context

The editor has a split-based window tree (`Window` enum) that supports `Directory` and `Tasks` leaf nodes. `:topen` creates a `Window::Tasks` in a horizontal split, with idempotent focus-switching and buffer refresh. The quickfix list (`QuickFix`) stores `Vec<PathBuf>` entries with a `current_index`, navigated by `:cfirst`/`:cn`/`:cN`. The `:cl` command prints entries to the command line but provides no persistent interactive view.

The copen window follows the established `Tasks` window pattern closely: same split creation, same navigation keymaps, same statusline structure, same buffer refresh mechanism.

## Goals / Non-Goals

**Goals:**
- Provide a persistent, navigable quickfix list window via `:copen`
- Allow jumping to entries (enter) and removing entries (dd)
- Keep the bold indicator in sync with `QuickFix.current_index`
- Reuse existing patterns (Tasks window, split creation, keymap handling)

**Non-Goals:**
- Editing quickfix entries inline (rename paths, reorder)
- Filtering or searching within the copen window
- Vertical split variant for copen
- Auto-opening copen when quickfix is populated

## Decisions

### 1. New `Window::QuickFix(ViewPort)` variant

Follow the `Window::Tasks(ViewPort)` pattern exactly. This requires updating all match arms on `Window` across the codebase (focused_viewport, focused_window_mut, buffer_ids, contains_tasks equivalent, etc.).

**Alternative**: Reuse `Window::Tasks` with a different buffer type. Rejected because the window type determines keymap routing and statusline rendering — conflating the two would require runtime type checks on the buffer.

### 2. New `Buffer::QuickFix(QuickFixBuffer)` variant

Similar to `Buffer::Tasks(TasksBuffer)`, holds a `TextBuffer` for the rendered lines. The `resolve_path` implementation returns `None` (same as Tasks).

**Alternative**: Store rendered lines directly in the QuickFix model. Rejected to maintain separation between data model (`QuickFix`) and view (`QuickFixBuffer`).

### 3. Bold rendering for current entry via ANSI escape codes

Use `\x1b[1m...\x1b[0m` (ANSI bold) embedded in `BufferLine` content for the entry matching `QuickFix.current_index`. This follows the existing pattern of ANSI-styled content (strikethrough for cancelled tasks, green for information).

**Alternative**: Use ratatui `Modifier::BOLD` at render time by checking index. Rejected because buffer content is rendered generically through the buffer view layer — ANSI embedding is the established mechanism for per-line styling in buffer content.

### 4. "Nearest directory window" resolution

When enter is pressed in the copen buffer, traverse the window tree to find the split containing the `QuickFix` window, then descend into the **other** child of that split to find the first `Window::Directory`. Algorithm:

1. Walk the tree from root, tracking the path to the `QuickFix` leaf
2. At the split containing QuickFix, take the sibling subtree
3. Recursively find the first `Directory` window in that sibling (follow focus path)
4. Navigate that directory window to the selected path

**Alternative**: Always navigate the globally focused directory window. Rejected because when copen is focused, the "focused directory" doesn't exist — we need spatial reasoning about the split layout.

### 5. Entry removal (dd) updates both QuickFix model and buffer

When dd is pressed:
1. Identify the entry at the cursor's `vertical_index`
2. Remove it from `QuickFix.entries`
3. Remove its sign from all directory buffers
4. Rebuild the copen buffer lines
5. Adjust cursor if it would overflow

This mirrors the sign cleanup in `qfix::toggle` and `qfix::reset`.

### 6. Keymap routing and cursor movement

The copen buffer accepts only:
- Navigation keys shared with `:topen` (j/k cursor movement, gg/G, etc.)
- `enter` — open selected entry in nearest directory window
- `dd` — remove selected entry from quickfix

All other keys are no-ops. This is enforced at the keymap layer by checking the focused window type, same as Tasks buffer handling.

Cursor movement (j/k) and viewport movement (gg/G) required changes to `cursor::relocate` and `viewport::relocate` which previously returned empty for Tasks and QuickFix buffers. Both now call `yeet_buffer::update` on the respective buffer's `TextBuffer` for these buffer types, without the directory-specific preview refresh logic.

### 9. Module structure for quickfix

The quickfix command code is organized as a module `update/command/qfix/` with:
- `commands.rs` — quickfix list commands (`:cfirst`, `:cn`, `:cN`, `:clearcl`, `:cdo`, `:invertcl`)
- `window.rs` — copen window logic (open, refresh, remove entry, nearest directory navigation)

### 7. Buffer refresh on quickfix navigation

When `:cfirst`, `:cn`, or `:cN` execute, after updating `current_index`, check if a QuickFix window exists and rebuild its buffer lines (updating which entry is bold). This parallels `refresh_tasks_buffer` being called after task state changes.

### 8. Idempotent open behavior

Like `:topen`, calling `:copen` when a QuickFix window already exists just switches focus to it (via `focus_quickfix` tree walk, mirroring `focus_tasks`).

## Risks / Trade-offs

- **Window enum expansion** — Every new leaf variant requires updating all match arms. This is mechanical but touches many files. → Mitigation: follow the Tasks pattern exactly, use compiler errors to find all sites.
- **Two special windows in one tab** — If both `:topen` and `:copen` are active, the split tree grows. → Mitigation: each creates its own horizontal split at the focused leaf, nesting naturally. No special handling needed.
- **Stale copen buffer** — If quickfix entries change via `:clearcl`, toggle, or `:cdo` while copen is open, the buffer could be stale. → Mitigation: call refresh after any quickfix mutation, same as task buffer refresh pattern.
- **`current_index` drift on dd** — Removing an entry before `current_index` shifts indices. → Mitigation: adjust `current_index` after removal (decrement if removed index < current_index, clamp if at end).
