## Context

Currently, `Message::QuickFixChanged` is emitted by 7 callers in `command/mod.rs` and `update/mod.rs` after calling qfix mutation functions. The handler in `update_with_message` iterates all tabs and calls `refresh_quickfix_buffer` for each. This spreads the responsibility across callers.

## Goals / Non-Goals

**Goals:**
- Each qfix command that mutates state emits `QuickFixChanged` itself.
- `refresh_quickfix_buffer` handles the cross-tab iteration internally.
- Callers no longer need to remember to emit the message.

**Non-Goals:**
- Changing behavior (refresh still happens across all tabs via emitted message).
- Changing `open.rs` direct calls (they must run synchronously before window mutations).

## Decisions

**1. Commands return `QuickFixChanged` emit in their actions.**

Each function in `commands.rs` (`select_first`, `next`, `previous`, `reset`, `clear_in`) and `qfix.rs` (`toggle`, `add`) will include `Action::EmitMessages(vec![Message::QuickFixChanged])` in their returned actions.

**2. `refresh_quickfix_buffer` takes tabs + contents and iterates all tabs.**

Change signature from `(window: &mut Window, contents: &mut Contents, qfix: &QuickFix)` to `(tabs: &mut HashMap<usize, Window>, contents: &mut Contents, qfix: &QuickFix)`. The function iterates all tabs internally, calling the per-window refresh logic for each.

Keep a private helper for the single-window refresh used by `open.rs` direct calls.

**3. Remove handler and emit calls from callers.**

- Remove `Message::QuickFixChanged` handler from `update_with_message`.
- Remove emit calls from `command/mod.rs` (5 sites) and `update/mod.rs` (2 sites).
- Remove `QuickFixChanged` from `Message` enum and Debug impl.

## Risks / Trade-offs

None. Pure refactor, same behavior.
