## Context

`wrap_line` builds continuation segments via `content.skip_chars(offset).take_chars(break_at)`. `skip_chars` uses `position_to_index` to find the byte offset of the Nth visible character and slices from there, discarding all preceding ANSI escape codes. This means if the original content is `"\x1b[31mhello world\x1b[0m"` and we wrap at 6 chars, the second segment starts at "world" without the `\x1b[31m` red foreground code.

## Goals / Non-Goals

**Goals:**

- Continuation segments carry the active ANSI styling from the wrap boundary.

**Non-Goals:**

- Optimizing for redundant escape codes (prepending all accumulated codes is acceptable even if some cancel out).

## Decisions

**Prepend accumulated ANSI escape sequences to continuation segments**

For each non-first segment, call `content.get_ansi_escape_sequences_till_char(offset)` to collect all ANSI codes that were active at the wrap point, then prepend them to the segment content. The first segment is unaffected since it starts at offset 0 and already has the original codes.

This is the simplest fix with zero risk to existing behavior — it only adds bytes to continuation segments, never removes anything.

## Risks / Trade-offs

- [Redundant escape codes in continuation segments] → Acceptable. The codes are invisible and `ansi_to_tui` handles stacked sequences correctly. No visual impact.
