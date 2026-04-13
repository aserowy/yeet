## Context

The buffer view renders each line by concatenating four prefix components (signs, line number, prefix column, border) followed by content. Each component is an `Ansi` string that may contain ANSI escape sequences for styling. The components are joined via `Ansi::join()` — plain string concatenation with no automatic resets between them.

Currently, some components emit ANSI escape codes but do not always terminate them cleanly:
- `get_signs()`: Emits a reset after each sign character, but only when signs are present. When no signs exist, it emits plain spaces (no ANSI).
- `get_line_number()` (cursor line): Emits `\x1b[1m` (bold) + fg color + text + `\x1b[0m\x1b[49m`. The reset is present.
- `get_line_number()` (non-cursor absolute): Emits plain text with **no ANSI codes at all** — the preceding ANSI state is inherited.
- `get_prefix_column()`: When a prefix is set, emits the raw prefix string (which may contain ANSI) + a reset. When no prefix is set, emits plain spaces (no ANSI).

The problem: when `ansi_to_tui` parses the concatenated string, characters from different components can end up in the same `Span` with inherited styling from the previous component. This causes the terminal to render certain characters (particularly PUA/nerdfont icons) differently depending on what ANSI state was active beforehand.

## Goals / Non-Goals

**Goals:**
- Ensure ANSI escape code styles do not bleed from one prefix component into the next
- Each component's ANSI styling must be self-contained
- Preserve existing rendering behavior for components that already render correctly

**Non-Goals:**
- Changing the visual layout or ordering of prefix components
- Changing the content width calculation or border width logic
- Fixing unicode-width measurement discrepancies (the icon should render as 1 cell when ANSI state is clean)

## Decisions

### Decision 1: Each prefix component that emits ANSI codes must end with a reset

Currently, cursor line numbers and prefix columns with content both terminate with `\x1b[0m\x1b[49m`. However, sign rendering does not always reset (when signs are present, each sign resets individually, but the padding spaces after are unstyled), and non-cursor absolute line numbers emit no ANSI at all.

**Approach**: Ensure every component that could have ANSI state active at its end emits an explicit `ansi_reset_with_bg(theme.buffer_bg)`. Components that emit only plain text (spaces) with no preceding ANSI codes can remain as-is.

Specifically:
- `get_signs()`: Already resets after each sign. The padding spaces inherit the reset state. No change needed.
- `get_line_number()` (cursor): Already resets. No change needed.
- `get_line_number()` (non-cursor relative): Already resets. No change needed.
- `get_line_number()` (non-cursor absolute): Emits NO ANSI codes. If signs preceded it with a reset, the state is clean. If signs were empty (just spaces), the state is whatever the terminal had. This is fine because no ANSI was emitted. No change needed.
- `get_prefix_column()` (with prefix): Already resets after prefix. No change needed.
- `get_prefix_column()` (no prefix): Emits plain spaces. No change needed.

### Decision 2: Components that contain styled content must prepend a reset before their content

The real issue is not about what happens at the END of a component, but at the START. When `get_prefix_column()` renders the icon, the icon is preceded by the line number's trailing escape codes. The `ansi_to_tui` parser groups the icon with the "reset" style from the line number's `\x1b[0m\x1b[49m]`. This creates a span with `fg=Reset, bg=Reset, not_bold, ...` — an **explicit** reset style that differs from the default "no style".

When ratatui renders this span, it must write explicit attribute-setting codes to the terminal (since the style has specific values). These codes, combined with the icon character, cause the terminal to misrender the icon's width.

**Fix**: The `get_prefix_column()` function should start its output with an explicit `\x1b[0m` to create a clean span boundary in `ansi_to_tui`. This ensures the icon character starts in a fresh ANSI context, isolated from the preceding line number styling.

**Alternative considered**: Adding resets between all components in the join chain in `get_styled_lines_nowrap`. Rejected because it would add unnecessary escape codes for components that don't need them and would change the span structure for non-prefix-column lines.

## Risks / Trade-offs

- [Risk] Adding `\x1b[0m` at the start of prefix column output creates an additional `ansi_to_tui` span boundary, slightly changing the span structure → Low impact since the visible rendering should remain identical, and ratatui handles multiple spans correctly.
- [Risk] The fix may not fully resolve the issue on all terminal emulators since PUA character rendering varies → The fix addresses the ANSI state bleeding, which is the confirmed root cause. Terminal-specific font rendering issues are out of scope.
