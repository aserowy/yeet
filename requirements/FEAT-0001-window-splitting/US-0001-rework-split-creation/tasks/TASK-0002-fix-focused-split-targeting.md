# Task: Fix Split Targeting For Focused Pane

## Metadata

- ID: TASK-0002
- Status: plan
- Userstory: US-0001

## Motivation

Splits must be created relative to the focused pane and applied to the most inner window in the tree. If split insertion targets the wrong node, users experience unpredictable layout changes.

## Relevant Acceptance Criteria

- Given a window with a focused pane
- When I create a split in a specified direction
- Then the split is created in that direction relative to the focused pane
- And the split is applied to the most inner window in the window tree

## Requirements

- Update split creation so it always replaces the focused leaf window (most inner focused pane), not a higher-level container.
- Preserve the requested split direction (horizontal vs vertical) relative to the focused pane.
- Ensure focus behavior after split remains consistent with existing split focus defaults unless explicitly required by the story.
- Add or update tests to cover focused-pane split targeting for both horizontal and vertical splits.

## Exclusions

- Do NOT change pane selection for targeted pane splits (handled in another task).
- Do NOT change unrelated window layout logic (sizing, rendering, or statusline).
- Do NOT alter keybindings or command parsing.

## Context

- @yeet-frontend/src/update/command/split.rs - split creation and insertion logic.
- @yeet-frontend/src/model/mod.rs - focused_window_mut, focused_viewport, and window tree structures.
- @yeet-frontend/src/update/app.rs - focused window helpers.
- @yeet-frontend/src/update/command/mod.rs - command dispatch for split/vsplit.
- @requirements/FEAT-0001-window-splitting/US-0001-rework-split-creation/story.md - acceptance criteria.
- @AGENTS.md - build/test/lint commands.

## Implementation Plan

### Step 1: Identify the focused leaf window target

Use existing helpers (e.g., `focused_window_mut`) or introduce a focused-leaf helper to obtain a mutable reference to the focused leaf window before replacing it with a split node.

```rust
let window = app.current_window_mut()?;
let target_leaf = window.focused_window_mut();
```

### Step 2: Replace focused leaf with split node

Update split creation to replace `target_leaf` with a split node that contains the old leaf and the new directory window in the correct orientation.

```rust
let old_window = std::mem::replace(target_leaf, Window::default());
*target_leaf = Window::Horizontal { first: Box::new(old_window), second: Box::new(new_window), focus: SplitFocus::First };
```

(Use the proper split focus behavior used today; keep parity with existing expectations.)

### Step 3: Update/add tests

Add tests for both `split` and `vsplit` that assert the split is inserted at the focused leaf (deepest pane), not at a higher-level node.

### Step 4: Run formatting and tests

Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test` per AGENTS.md.

## Examples

- Given a nested split with focus on the second child of the first branch, `split` should wrap only that focused leaf, leaving the rest of the tree unchanged.
