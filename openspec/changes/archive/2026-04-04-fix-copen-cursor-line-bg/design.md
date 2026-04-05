## Context

The rendering pipeline in `yeet-buffer` builds styled lines as ANSI strings, then converts them to ratatui `Line` objects via `ansi_to_tui`. For the cursor line, `add_cursor_styles` in `line.rs` prepends the cursor line background ANSI code and appends a reset-to-buffer-bg at the end.

The quickfix current item is bold-formatted using `\x1b[1m...\x1b[0m`. The `\x1b[0m` is a full ANSI reset that clears **all** attributes, including the background color. When this reset appears mid-line on the cursor line, the cursor line background is lost for the remainder of the text — `ansi_to_tui` faithfully interprets the reset, dropping the background.

## Goals / Non-Goals

**Goals:**
- Ensure cursor line background is preserved through embedded ANSI resets within cursor line content.

**Non-Goals:**
- Changing how the current qfix item is indicated (bold stays).
- Changing the overall ANSI rendering architecture.

## Decisions

**Replace embedded resets with cursor-line-aware resets on cursor lines.**

In `add_cursor_styles`, after prepending the cursor line background, replace any `\x1b[0m` sequences within the content with `\x1b[0m` followed by the cursor line background code. This ensures that after any ANSI reset within the line, the cursor line background is immediately re-applied.

Alternative considered: Modifying `ansi_to_tui` or post-processing the ratatui `Line` spans. Rejected because the ANSI string layer is where the styling is composed — fixing it there is simpler and keeps the conversion library as a pure pass-through.

Alternative considered: Removing the `\x1b[0m` from the bold formatting in `build_qfix_lines` and using a more targeted reset (e.g., `\x1b[22m` for bold-off only). This would fix the specific case but not protect against other ANSI-styled content (e.g., strikethrough for cancelled tasks) that also uses full resets.

## Risks / Trade-offs

[Broader impact] The fix applies to all cursor lines with embedded resets, not just the qfix window. This is correct behavior — any buffer line with ANSI resets should maintain cursor line background when on the cursor line. → No mitigation needed; this is desirable.
