## Context

The `highlight_markdown` function uses `syntect`'s `LinesWithEndings` iterator, which preserves trailing `\n` on each line. After ANSI highlighting, a line like `"# Commands\n"` becomes `"\x1b[...m# Commands\n\x1b[0m"`. The trailing `\n` is embedded inside the ANSI-escaped string.

When this goes through the rendering pipeline:
1. `Ansi::count_chars()` counts the `\n` as a visible character (it's not an ANSI escape)
2. Padding calculation is off by 1 (`content_width - line_length` is 1 less than needed)
3. `ansi_to_tui::IntoText` splits on `\n`, producing 2 `ratatui::Line`s per buffer line
4. The second empty line gets cursor line background, creating the visual wrap

## Goals / Non-Goals

**Goals:**

- Ensure each syntax-highlighted line produces exactly one `BufferLine`, regardless of embedded newlines from `LinesWithEndings`.

**Non-Goals:**

- Changing the `Ansi` struct or the `ansi_to_tui` conversion pipeline.
- Fixing this generically for all ANSI content — only help buffer content has this issue.

## Decisions

**Split highlighted strings on `\n` into separate `BufferLine`s using `split_terminator`**

Replace the current `.map(|l| BufferLine::from(l.as_str()))` with `.flat_map(|l| l.split_terminator('\n').map(BufferLine::from))`. This splits any highlighted string containing newlines into multiple `BufferLine`s. `split_terminator` treats `\n` as a line terminator (not a separator), so `"# Commands\n"` produces `["# Commands"]` — no trailing empty string.

Alternative considered: Trimming trailing newlines with `trim_end_matches('\n')`. Rejected because it silently drops content rather than preserving it. If syntect ever produces a genuinely multi-line highlighted string, trimming would lose content while splitting preserves it as separate buffer lines.

Alternative considered: Using `content.lines()` instead of `LinesWithEndings::from(content)` in `highlight_markdown`. Rejected because `syntect`'s `HighlightLines::highlight_line` expects lines from `LinesWithEndings` for correct stateful highlighting across lines.

## Risks / Trade-offs

- [Extra empty lines from splitting] → Mitigated by using `split_terminator` which does not produce a trailing empty segment for a terminating `\n`. A `\n\n` in the middle would correctly produce an empty `BufferLine` representing a blank line.
