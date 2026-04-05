## Context

The `Window` enum has `Tasks(ViewPort)` and `QuickFix(ViewPort)` variants that often have identical match arm bodies. Most places in the codebase already combine them with `|` patterns, but 4 spots in `model/mod.rs` still have separate arms.

## Goals / Non-Goals

**Goals:**
- Combine the 4 remaining duplicate match arms in `model/mod.rs`.

**Non-Goals:**
- Combining arms that have different bodies (e.g., `contains_tasks`, `contains_quickfix`, tab titles).

## Decisions

Use `Window::QuickFix(vp) | Window::Tasks(vp) =>` pattern for the 4 sites:
- `focused_viewport` (line ~172): both return `vp`
- `focused_window_mut` (line ~193): both return `self`
- `focused_viewport_mut` (line ~200): both return `vp`
- `buffer_ids` (line ~219): both return `HashSet::from([vp.buffer_id])`

## Risks / Trade-offs

None. Pure refactor, no behavior change.
