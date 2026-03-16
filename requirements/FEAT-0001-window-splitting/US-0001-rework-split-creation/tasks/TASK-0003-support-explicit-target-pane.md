# Task: Support Explicit Target Pane For Split Creation

## Metadata

- ID: TASK-0003
- Status: plan
- Userstory: US-0001

## Motivation

When users select a specific target pane, split creation must apply to that pane instead of whichever pane is currently focused. This ensures predictable behavior in multi-pane layouts.

## Relevant Acceptance Criteria

- Given multiple panes are visible
- When I select a target pane and create a split
- Then the new split is attached to the selected pane rather than a different pane
- And the split is applied to the selected pane's most inner window in the window tree

## Requirements

- Provide a reliable way to resolve an explicit target pane into the correct leaf window in the tree.
- When a target pane is specified, use it instead of focused pane selection.
- Preserve the split direction relative to the target pane (horizontal vs vertical).
- Add or update tests that validate target-pane split insertion in both horizontal and vertical directions.

## Exclusions

- Do NOT alter focused-pane splitting behavior (handled in another task).
- Do NOT change the command syntax unless required by current APIs.
- Do NOT adjust split sizing, layout rendering, or focus navigation beyond insertion.

## Context

- @yeet-frontend/src/update/command/split.rs - split creation and insertion logic.
- @yeet-frontend/src/update/command/mod.rs - how split commands parse targets (if any).
- @yeet-frontend/src/model/mod.rs - window tree and focus/viewport accessors.
- @yeet-frontend/src/update/app.rs - window lookup helpers.
- @requirements/FEAT-0001-window-splitting/US-0001-rework-split-creation/story.md - acceptance criteria.
- @AGENTS.md - build/test/lint commands.

## Implementation Plan

### Step 1: Identify how target pane selection is represented

Locate where a “selected pane” is tracked (e.g., via viewport IDs or selection state). Document the existing representation and how it can be resolved to a leaf window.

### Step 2: Implement target-pane resolution helper

Add or reuse a helper that traverses the window tree and returns a mutable reference to the matching leaf window for a given pane identifier.

```rust
fn find_window_by_viewport_id(window: &mut Window, target_id: usize) -> Option<&mut Window> {
    // traverse and return leaf
}
```

### Step 3: Use target pane when specified

Update split creation to prefer the target pane over focused pane when a target is provided. Ensure the split replaces the target leaf window.

### Step 4: Add tests

Cover a nested layout where focus is on a different pane than the target. Verify the split attaches to the target pane’s leaf window and direction.

### Step 5: Run formatting and tests

Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test`.

## Examples

- Layout: Horizontal split; focus on left, target right. `vsplit` should split only the right pane’s leaf window, not the focused left pane.
