## Context

`remove_entry` in `qfix/window.rs` guards with `if cursor_index >= qfix.entries.len() { return Vec::new(); }`. After removing the last entry in the list, the viewport cursor can be at an index equal to the new entry count. Subsequent `dd` hits this guard and does nothing.

## Goals / Non-Goals

**Goals:**
- `dd` with an out-of-bounds cursor clamps to the last entry and removes it.

**Non-Goals:**
- Changing cursor clamping in `refresh_quickfix_buffer` (that already works correctly for the viewport cursor after removal).

## Decisions

Change the guard in `remove_entry` to clamp `cursor_index` to `entries.len() - 1` when it exceeds bounds, rather than returning early. Keep the early return only for the empty entries case.

## Risks / Trade-offs

None. This is a simple bounds-clamping fix.
