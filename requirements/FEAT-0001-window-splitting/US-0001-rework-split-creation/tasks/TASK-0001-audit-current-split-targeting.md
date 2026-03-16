# Task: Audit Current Split Targeting Behavior

## Metadata

- ID: TASK-0001
- Status: done
- Userstory: US-0001

## Motivation

We need a clear understanding of how split targeting and focus resolution works today before changing behavior. This audit prevents unintended regressions and ensures subsequent tasks implement the intended pane selection and direction logic.

## Relevant Acceptance Criteria

- Given a window with a focused pane
- When I create a split in a specified direction
- Then the split is created in that direction relative to the focused pane
- And the split is applied to the most inner window in the window tree

## Requirements

- Document the current split creation flow end-to-end, including how direction, focus, and target pane are determined.
- Identify the exact functions that choose the target window/pane for split insertion.
- Capture current behavior for: focused pane split, targeted pane split, and nested split insertion.
- Summarize mismatches between current behavior and acceptance criteria in a short checklist for follow-up tasks.

## Exclusions

- Do NOT change any production code in this task.
- Do NOT add or modify tests in this task.
- Do NOT refactor unrelated window or command logic.

## Context

- @yeet-frontend/src/update/command/split.rs - split creation entry points and helpers.
- @yeet-frontend/src/model/mod.rs - window tree, focus, and viewport accessors.
- @yeet-frontend/src/update/app.rs - helpers that traverse focused windows.
- @yeet-frontend/src/update/command/mod.rs - command routing for split/vsplit.
- @requirements/FEAT-0001-window-splitting/US-0001-rework-split-creation/story.md - acceptance criteria.

## Implementation Plan

### Step 1: Trace split command routing

Read the split command handlers to see how direction and target path are passed into the split creation helper. Capture the call chain and note any target pane selection logic.

### Step 2: Trace window targeting

Follow the helper(s) that choose which window/pane in the tree is replaced by the new split. Note how focus, selected pane, and "most inner window" are interpreted in the current code.

### Step 3: Summarize behavior vs criteria

Write a short checklist (bulleted) in the task notes for mismatches against the acceptance criteria, including any ambiguity that needs confirmation.

## Examples

- Example checklist item: "Focused pane is resolved via focused_viewport(), but split insertion replaces the top-level window instead of the focused leaf." (Replace with actual findings.)

## Notes

- Command routing: `update/command/mod.rs` maps `split` -> `split::horizontal` and `vsplit` -> `split::vertical`, passing a resolved target path based on the focused current buffer.
- Split creation entry point: `update/command/split.rs::create_split` obtains `app.current_window_mut()` and uses `mem::take` to replace the *entire current window tree* with a new split containing the old tree and a new directory window.
- Target selection: no focused leaf selection is used; `Window::focused_window_mut` exists in `model/mod.rs` but is not referenced by split creation.
- Focus resolution: `Window::focused_viewport()` follows `SplitFocus` recursively, but split insertion ignores this and always wraps the root window.

### Behavior checkpoints

- Focused pane split: split wraps the root window, not the focused leaf.
- Targeted pane split: no dedicated targeting logic; only current window is replaced.
- Nested split insertion: even when focus is deep, split replaces the top-level window tree.

### Mismatches vs acceptance criteria

- [ ] Split is not applied to the most inner focused window; it replaces the top-level window tree.
- [ ] Direction is applied at the root, not relative to the focused leaf.
