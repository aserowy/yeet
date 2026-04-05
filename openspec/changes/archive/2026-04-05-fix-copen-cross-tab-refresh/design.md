## Context

QuickFix state (`state.qfix`) is global across tabs — there's one quickfix list shared by all tabs. The copen buffer, however, lives in a specific tab's window tree. Commands like `:cfirst` modify the global qfix state and then call `refresh_quickfix_buffer` on `app.current_window_and_contents_mut()`, which only returns the current tab. Additionally, `buffers::update` runs every cycle and removes `Buffer::QuickFix` entries not referenced by the current tab's window tree.

## Goals / Non-Goals

**Goals:**
- QuickFix buffer cleanup considers all tabs, not just the current one.
- QuickFix buffer refresh after mutations works regardless of which tab is active.
- Decouple buffer refresh from command handlers via a new `QuickFixChanged` message.

**Non-Goals:**
- Changing how `remove_entry` or `open` refresh the buffer (these always operate on the current tab where copen is focused, so direct calls are fine).

## Decisions

**1. Fix buffer cleanup to check all tabs.**

In `buffers::update`, replace `app.current_window().buffer_ids()` with collecting buffer IDs from `app.tabs.values()`. This prevents cross-tab buffer removal.

**2. Add `Message::QuickFixChanged` and emit it instead of calling `refresh_quickfix_buffer` directly.**

Production call sites to replace (10 total, replace 8):
- `update/command/mod.rs`: cfirst, clearcl (x2), cn, cN (5 sites)
- `update/mod.rs`: FdResult/RgResult handler, ToggleQuickFix handler (2 sites)
- `update/open.rs`: copen enter handler (1 site)

Keep direct calls in:
- `qfix/window.rs::remove_entry` — always operates on the current tab's copen
- `qfix/window.rs::open` — internal, called during copen creation on current tab

**3. Handle `Message::QuickFixChanged` in `update_with_message`.**

Iterate `app.tabs.values_mut()` and call `refresh_quickfix_buffer` for each tab's window tree. Use `app.contents` for the shared buffer store.

## Risks / Trade-offs

[Message ordering] `QuickFixChanged` is emitted via `Action::EmitMessages`, which processes in the next update cycle. The qfix state is already mutated by the time the message is handled, so the refresh will see the correct state. → No mitigation needed.

[Performance] Iterating all tabs on every qfix mutation is negligible — tabs are typically few (< 10). → No mitigation needed.
