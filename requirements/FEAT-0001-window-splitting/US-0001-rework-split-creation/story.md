# User Story: Rework Split Creation Flow

## Metadata

- ID: US-0001
- Status: plan
- Feature: FEAT-0001
- As a: user

## Capability

I want: split creation to consistently target the intended pane and direction

## Benefit

So that: I can set up my workspace layout without surprises

## Acceptance Criteria

- Given a window with a focused pane
- When I create a split in a specified direction
- Then the split is created in that direction relative to the focused pane
- And the split is applied to the most inner window in the window tree
- Given multiple panes are visible
- When I select a target pane and create a split
- Then the new split is attached to the selected pane rather than a different pane
- And the split is applied to the selected pane's most inner window in the window tree
