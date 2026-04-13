## Context

The yeet TUI file manager uses chafa as a fallback image renderer when native terminal image protocols (Kitty, Sixel, iTerm2, halfblocks via `ratatui-image`) are unavailable. The `load_with_chafa` function in `yeet-frontend/src/task/image.rs` spawns chafa with `--view-size {width}x{height}` using the raw preview viewport dimensions (`rect.width`, `rect.height`).

The chafa output (ANSI-encoded Unicode symbol art) is captured as lines of text, stored as `Preview::Content`, and rendered through the standard buffer view pipeline in `yeet-buffer`. This pipeline adds sign columns, line numbers, prefix columns, and borders — all of which reduce the actual renderable content area below the raw viewport width.

Currently there are two bugs:

1. **Viewport width mismatch**: The raw `preview_vp.width` is passed to chafa, but the buffer rendering pipeline may subtract offset width from the available content area. While the preview viewport defaults have zero offsets (no sign columns, no line numbers, no prefix columns, no border), if any of these are ever set (e.g., via plugins or configuration changes), the chafa output will overflow. Additionally, the preview pane in a vertical split context can have `show_border: true` forced, which reduces the inner rect by 1 column via `Block::inner()`.

2. **Non-SGR escape sequences**: The `Ansi` type in `yeet-buffer/src/model/ansi.rs` only recognizes escape sequences ending in `m` (SGR sequences for colors/styles). Chafa with `-f symbols --relative off` can emit non-SGR CSI sequences (e.g., cursor positioning `\x1b[...H`, cursor movement `\x1b[...C`, erase sequences `\x1b[...J`). When the `Ansi` parser encounters these, it waits for an `m` that never comes (or comes much later), treating intermediate characters as part of an escape and corrupting the visible output. The `ansi_to_tui` crate may also interpret these sequences differently, causing raw escape code text to appear.

## Goals / Non-Goals

**Goals:**
- Fix chafa output to correctly fit within the preview viewport's renderable content area
- Eliminate visible escape codes in chafa-rendered image previews
- Ensure the fix works for both default preview viewports and edge cases (vertical splits with borders)

**Non-Goals:**
- Overhauling the `Ansi` type to be a full ANSI/VT terminal escape sequence parser — that's a larger refactor
- Supporting animated chafa output
- Changing the chafa command-line flags beyond what's needed for the fix (e.g., not switching from `-f symbols` to another format)

## Decisions

### 1. Compute content-area dimensions before calling chafa

**Decision**: Calculate the effective content width/height by accounting for viewport offsets (sign columns, line numbers, prefix columns, border) before passing dimensions to chafa's `--view-size`.

**Rationale**: The buffer view pipeline will always consume offset columns when rendering. Passing raw viewport dimensions to chafa means the image is sized for a wider area than what's actually available. By pre-computing the content area, chafa produces output that fits exactly.

**Alternative considered**: Post-processing chafa output to truncate lines — rejected because it would cut through multi-byte Unicode characters and mid-ANSI escape sequences, producing corrupted output.

### 2. Sanitize chafa output by stripping non-SGR escape sequences

**Decision**: After capturing chafa's stdout, strip all CSI escape sequences that do not end in `m` before storing the lines as `Preview::Content`. This is done in `load_with_chafa` at the point where stdout is converted to lines.

**Rationale**: The downstream `Ansi` type and `ansi_to_tui` crate are designed for SGR (color/style) sequences only. Non-SGR sequences (cursor movement, erase, etc.) have no meaning in a ratatui `Paragraph` widget and would produce visual artifacts. Stripping them at the source is the cleanest approach.

**Alternative considered**: Making the `Ansi` type handle all CSI sequences — rejected as a much larger change with broader impact across the codebase, and the non-SGR sequences are meaningless in the rendering context anyway.

### 3. Pass viewport context to the preview load task

**Decision**: Modify the `Task::LoadPreview` to receive the content-area rect (with offsets already subtracted) rather than the raw viewport rect. The computation happens in `action.rs` where the preview viewport is available.

**Alternative considered**: Passing the full `ViewPort` to the task and computing content width inside `load_with_chafa` — rejected because it would require the task module to depend on `ViewPort` internals and a dummy `BufferLine` to compute content width. Computing it at the call site where the viewport context is naturally available is cleaner.

## Risks / Trade-offs

- **[Risk] Preview viewport offsets change dynamically** → The content-area rect is computed at preview-load time. If viewport offsets change between loading and rendering, there could be a brief size mismatch. This is acceptable as the preview is reloaded on navigation. → Mitigation: reloading on resize already handles this.
- **[Risk] Stripping non-SGR sequences could remove intentional formatting** → Chafa's `-f symbols` mode primarily uses SGR for colors. Cursor movement sequences are artifacts of `--relative off` mode and are not needed for line-by-line rendering. → Mitigation: only strip CSI sequences; OSC and other escape types are left alone (though chafa doesn't typically emit them).
- **[Trade-off] The `Ansi` type still has fragile escape parsing** → This fix doesn't address the fundamental limitation. A future refactor could make `Ansi` handle all CSI sequences properly, but that's out of scope.
