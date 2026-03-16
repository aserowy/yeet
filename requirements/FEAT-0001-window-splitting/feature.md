# Feature: Window Splitting

## Metadata

- ID: FEAT-0001
- Status: plan

## Summary

Enable users to split the window into multiple panes and manage those splits for side-by-side or stacked workflows.

## Scope

- Provide split creation in at least two directions (vertical and horizontal).
- Ensure the split targets the focused pane and applies to the most inner window in the window tree.
- Support nested splits so users can iteratively split panes within existing splits.
- Ensure split creation integrates with the existing window/pane model without regressions.

## Out of Scope

- Advanced layout persistence or custom layout templates.
- Keyboard remapping for split commands beyond current defaults.
- Auto-balancing or auto-resizing splits beyond current behavior.

## User Stories

- US-0001 rework split creation flow

## Notes

- Focus on reworking the current split creation to be consistent, predictable, and compatible with existing behavior.
