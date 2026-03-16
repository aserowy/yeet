# Quality Gate: US-0001 Rework Split Creation Flow

## Gate Checks

- [ ] Tasks are non-overlapping and have clear ownership of focused-pane vs target-pane logic.
- [ ] Split targeting changes specify "most inner window" behavior explicitly.
- [ ] Tests cover both horizontal and vertical splits.
- [ ] Tests include nested split scenarios.
- [ ] No task changes layout rendering, keybindings, or unrelated window sizing.

## Conflict Review

- TASK-0002 handles focused-pane split targeting only.
- TASK-0003 handles explicit target-pane split targeting only.
- TASK-0004 handles regression testing only; it must not change production logic.

## Exit Criteria

- All tasks remain in plan status before implementation starts.
- Task IDs and slugs are unique and consistent with directory conventions.
