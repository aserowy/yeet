## Context

Yeet's file preview renders non-image files through a syntax highlighting pipeline: `syntect` highlights the file content into ANSI escape sequences, which are stored as `Ansi` structs in `BufferLine`s, then rendered via `ansi_to_tui::IntoText` into ratatui `Line`/`Span` objects.

The current pipeline has no sanitization or size limits for syntax-highlighted content. A file like `assets/logo.svg` contains a 517K-character line (base64-encoded PNG data). When syntect highlights this line, it wraps every token in `\x1b[38;2;R;G;Bm...\x1b[0m` sequences, producing a multi-megabyte ANSI string. This pathological string, when processed through `ansi_to_tui` and rendered by ratatui's crossterm backend, generates terminal output that causes the terminal emulator to exit raw mode or alternate screen. The result: the preview appears empty, keyboard input bypasses yeet entirely, and after killing yeet, accumulated input is executed in the shell — a remote code execution vector via crafted files.

Notably, the chafa image preview path already has a `strip_non_sgr_escape_sequences()` function that sanitizes its output. The syntax highlight path lacks equivalent protection.

## Goals / Non-Goals

**Goals:**
- Prevent syntax-highlighted preview content from escaping the terminal/TUI by sanitizing highlight output
- Truncate extremely long lines before highlighting to prevent pathological ANSI string generation
- Share the existing sanitization logic between chafa and syntax highlight paths
- Maintain existing behavior: SVG files are still previewed as highlighted text (not images)

**Non-Goals:**
- Changing how `infer` detects MIME types (SVGs will continue to be treated as text)
- Improving SVG rendering or adding SVG-specific preview handling
- Modifying the `ansi_to_tui` or `syntect` crates themselves

## Decisions

### Decision 1: Truncate lines before highlighting

Lines exceeding a maximum character length will be truncated before being passed to syntect's highlighter. This prevents syntect from generating pathological multi-megabyte ANSI strings.

**Rationale**: The root cause is that a 517K character line becomes a multi-megabyte ANSI string after highlighting. Truncating before highlighting avoids the problem at the source. A truncated line with a "…" indicator is a reasonable UX for preview — users aren't editing in the preview pane.

**Alternative considered**: Truncating after highlighting — rejected because the ANSI string is already generated and parsing it to find a safe truncation point that doesn't break escape sequences is more complex and wasteful.

**Limit**: Use the viewport's content width as the truncation point, since lines beyond the visible area are not displayed anyway. This makes the truncation invisible to the user in practice.

### Decision 2: Apply non-SGR escape sequence stripping to syntax highlight output

The existing `strip_non_sgr_escape_sequences()` function (currently in `image.rs` for chafa output) will also be applied to syntax-highlighted lines. This ensures that even if syntect or `as_24_bit_terminal_escaped()` produces non-SGR CSI sequences (cursor movement, erase, etc.), they are stripped before entering the buffer pipeline.

**Rationale**: Defense in depth. While syntect's `as_24_bit_terminal_escaped()` should only produce SGR sequences, we cannot guarantee this for all inputs, especially adversarial ones. The chafa path already demonstrates this pattern is necessary.

**Alternative considered**: Only truncating lines without sanitizing — rejected because truncation alone doesn't protect against crafted content that embeds terminal escape sequences within shorter lines.

### Decision 3: Extract sanitization to a shared module

Move `strip_non_sgr_escape_sequences()` from `yeet-frontend/src/task/image.rs` to a new `yeet-frontend/src/task/sanitize.rs` module, then use it from both `image.rs` and `syntax.rs`.

**Rationale**: Code reuse, single source of truth for sanitization logic. Both preview paths need the same protection.

**Alternative considered**: Duplicating the function in `syntax.rs` — rejected to avoid maintenance burden.

### Decision 4: Pass content width to the highlight function

The `highlight()` function in `syntax.rs` will accept a `content_width` parameter (derived from the preview viewport's content area width). This is used to truncate lines before highlighting.

**Rationale**: The preview viewport already computes content-area dimensions for chafa (per the existing `chafa` spec). Using the same value for syntax highlighting ensures truncation is consistent with what the user can see.

## Risks / Trade-offs

- **[Risk] Truncation may hide content in wrapped mode** → When wrap is enabled, truncated content won't wrap because the line is shorter than the viewport width. This is acceptable for preview buffers which are read-only and typically viewed without wrap.
- **[Risk] Extremely wide terminals may still produce large lines** → Even with a 300-column terminal, a single highlighted line would produce at most ~10KB of ANSI, which is safe. The risk is only with extremely long source lines.
- **[Risk] Tests for `strip_non_sgr_escape_sequences` move to new module** → Existing tests in `image.rs` must be moved to `sanitize.rs`. This is mechanical but must be verified.
