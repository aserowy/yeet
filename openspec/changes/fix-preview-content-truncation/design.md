## Context

The security fix in commit `79b2ecb` introduced `truncate_line()` in `yeet-frontend/src/task/syntax.rs` to prevent pathologically long lines (e.g., 517K-char base64 in SVGs) from generating multi-megabyte ANSI strings that broke the terminal. The truncation happens **before** syntect highlighting: each line is cut to `content_width` characters, then passed to syntect.

Syntect's `HighlightLines` is a stateful parser — it carries parser state across lines. When a line like `src="https://...long-url..."` is truncated mid-string-literal, the closing `"` is lost. Syntect then enters the next line still "inside a string," and this state cascades through all subsequent lines, coloring everything as a string literal until a coincidental `"` resets the parser.

The call site at `task/mod.rs:492` passes `rect.width` (full viewport width) as the truncation limit. The view layer already handles display-width fitting: `skip_chars` for horizontal scrolling (nowrap mode) and word wrapping (wrap mode). The `Ansi` model preserves full line content and the view layer truncates/wraps at render time.

## Goals / Non-Goals

**Goals:**

- Fix the style-bleed bug: syntect must receive the full original line so its cross-line parser state is maintained correctly
- Preserve the non-SGR escape sequence sanitization (the actual security fix from `strip_non_sgr_escape_sequences`)
- Let the view layer handle display-width truncation as it already does for all other buffer content

**Non-Goals:**

- Adding a pre-highlight safety cap (the user explicitly decided against this)
- Changing the view-layer rendering pipeline or `Ansi` model
- Modifying the wrap/nowrap rendering logic in `yeet-buffer`
- Adding new syntax highlighting backends or replacing syntect

## Decisions

### Decision 1: Remove pre-highlight line truncation entirely

**Approach:** Remove the `truncate_line()` function and all pre-highlight truncation from `syntax.rs`. Pass the full original line to syntect for highlighting, preserving correct parser state across all lines.

**Rationale:**
- The view layer already handles display-width fitting via `skip_chars` (horizontal scrolling in nowrap mode) and word wrapping. Long lines in buffers are stored at full width and truncated/wrapped at render time.
- Pre-highlight truncation was the root cause of the style-bleed bug. Removing it fixes the bug at its source.
- The `strip_non_sgr_escape_sequences` sanitization remains — this is the actual protection against terminal breakout from non-SGR escape sequences in highlighted output.

### Decision 2: Remove `content_width` parameter from `highlight()`

Since `highlight()` no longer needs to know the viewport width (it returns full-width highlighted lines), the `content_width` parameter should be removed from the function signature and from the call site at `task/mod.rs:492`.

### Decision 3: Rely on existing view-layer truncation

The full highlighted ANSI content is stored in `BufferLine.content` as an `Ansi` object. The view layer handles display:
- **Nowrap mode:** `add_line_styles()` calls `skip_chars(vp.horizontal_index)` for horizontal scrolling; ratatui clips overflow at the widget boundary
- **Wrap mode:** `add_line_styles_wrap()` handles word wrapping across visual lines with ANSI style preservation via `get_ansi_escape_sequences_till_char()`

No changes are needed in the view layer.

## Risks / Trade-offs

- **[Risk] Pathologically long lines (500K+ chars) will generate large ANSI output from syntect** → Mitigated by `strip_non_sgr_escape_sequences` which removes non-SGR escapes. The original terminal breakout was caused by non-SGR sequences (cursor movement, erase), not by SGR color codes. The sanitization addresses the actual vulnerability. Performance impact of large ANSI strings is acceptable — syntect processes the content in a background task, and the view layer's `Ansi` methods handle large strings efficiently.
- **[Risk] Very large highlighted strings consume more memory** → Acceptable trade-off for correctness. Files with extremely long lines are rare and the memory overhead is bounded (syntect output is proportional to input size).
