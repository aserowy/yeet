## Why

The statusline view functions for Help, Tasks, and QuickFix are nearly identical — each has a focused and unfocused variant that differs only in the label string (`"Help"`, `"Tasks"`, `"QuickFix"`) and the buffer type used to extract `lines.len()`. This results in 6 functions (~120 lines) that could be 2 generic functions (~40 lines).

## What Changes

- Replace `help_status`, `tasks_status`, `quickfix_status` with a single `label_status` function that takes the label string, line count, viewport, frame, rect, and theme.
- Replace `help_status_unfocused`, `tasks_status_unfocused`, `quickfix_status_unfocused` with a single `label_status_unfocused` function that takes the label string, frame, rect, and theme.
- Update the `view` match arms to extract the line count from each buffer type and call the shared functions.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none — pure refactor, no behavior change)

## Impact

- `yeet-frontend/src/view/statusline.rs`: Remove 6 functions, add 2 generic replacements, update match arms.
