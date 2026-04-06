## Why

When a styled line wraps into multiple visual lines, continuation segments lose the ANSI styling from the first segment. This happens because `Ansi::skip_chars` drops all escape sequences before the skip offset, so continuation segments start with no active styling. For example, a red-colored line that wraps shows the first visual line in red but subsequent lines in the default terminal color.

## What Changes

- In `wrap_line`, prepend the accumulated ANSI escape sequences from the original content up to each segment's start offset. This carries the active styling context into each continuation segment.
- The existing `Ansi::get_ansi_escape_sequences_till_char` method already collects all ANSI codes up to a given visible character position — use it to extract the style prefix for each continuation segment.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `buffer`: Wrapped continuation lines SHALL preserve the ANSI styling active at the wrap boundary.

## Impact

- `yeet-buffer/src/view/wrap.rs`: Prepend style prefix to continuation segment content in `wrap_line`.
