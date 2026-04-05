## Context

`open.rs` has two `refresh_quickfix_buffer_in_window` calls — one in the existing-sibling branch and one in the create-split branch. Both run synchronously before window tree mutations (focus change, split creation). The refresh updates bold formatting based on `current_index`. Since the `NavigateToPathAsPreview` is already emitted as an async action, the refresh can also be async via `Message::QuickFixChanged`.

## Goals / Non-Goals

**Goals:**
- Replace direct `refresh_quickfix_buffer_in_window` calls in `open.rs` with `Message::QuickFixChanged` emit.
- Make `refresh_quickfix_buffer_in_window` private since no external callers remain.

**Non-Goals:**
- Changing any other refresh call sites.

## Decisions

Append `Action::EmitMessages(vec![Message::QuickFixChanged])` to the returned actions vector alongside the existing `NavigateToPathAsPreview` emit. Remove the direct calls and the `current_window_and_contents_mut` borrows that existed solely for the refresh.

## Risks / Trade-offs

None. The refresh will execute in the next update cycle, same as `NavigateToPathAsPreview`.
