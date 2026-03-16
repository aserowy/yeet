# Task: Add Regression Tests For Split Targeting

## Metadata

- ID: TASK-0004
- Status: plan
- Userstory: US-0001

## Motivation

Regression tests ensure the new split targeting logic stays correct over time and catches accidental changes that break pane selection or direction.

## Relevant Acceptance Criteria

- Then the split is created in that direction relative to the focused pane
- And the split is applied to the most inner window in the window tree
- Then the new split is attached to the selected pane rather than a different pane

## Requirements

- Add tests that validate split insertion for both focused and explicitly targeted panes.
- Include nested split scenarios to ensure insertion is at the most inner window.
- Verify the orientation of the created split matches the command direction (horizontal vs vertical).
- Keep tests focused and deterministic.

## Exclusions

- Do NOT change production split logic in this task.
- Do NOT add new features or command syntax changes.
- Do NOT modify unrelated tests.

## Context

- @yeet-frontend/src/update/command/split.rs - existing split tests and helpers.
- @yeet-frontend/src/model/mod.rs - window tree and focus types for asserting structure.
- @AGENTS.md - build/test/lint commands.

## Implementation Plan

### Step 1: Review existing split tests

Identify current tests in `split.rs` and note which scenarios are missing for focused vs target panes and nested layouts.

### Step 2: Add nested layout fixtures

Add helper builders in tests if needed to construct nested splits with specific focus and target selection.

### Step 3: Add regression cases

Add tests that assert:

- Focused pane splitting replaces the focused leaf only.
- Targeted pane splitting replaces the target leaf only (even when focus differs).
- The resulting split orientation matches the command.

### Step 4: Run formatting and tests

Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test`.

## Examples

- A vertical split command on a targeted leaf produces `Window::Vertical` at that leaf with the old and new panes as children.
