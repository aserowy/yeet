## Context

The buffer viewport layout reserves a 1-cell border space between pre-content columns (signs, line numbers) and the content area. This border is controlled by `get_border_width()`, which currently returns 1 only when `get_prefix_width() > 0`. The `get_prefix_width()` function sums `sign_column_width + line_number_width` — it does not include `prefix_column_width`.

The `prefix_column_width` is a separate column that sits after the border in the layout:
```
[signs][line_number][border][prefix_column][content]
```

When signs or line numbers are active, the border is drawn and the prefix column appears between the border and content. However, when only the prefix column is active (no signs, no line numbers), `get_prefix_width()` returns 0, `get_border_width()` returns 0, and the layout becomes:
```
[prefix_column][content]  (no border — icon glued to filename)
```

## Goals / Non-Goals

**Goals:**
- Ensure the 1-cell border space is rendered whenever any pre-content column is active, including when only the prefix column is set
- Maintain correct content width calculations so lines still fill the viewport exactly

**Non-Goals:**
- Changing the visual appearance of the border (it remains a single space)
- Changing the column ordering (signs → line numbers → border → prefix column → content)
- Adding configurable border styling

## Decisions

### Decision 1: Include prefix column width in the border condition

Change `get_border_width()` to check whether any pre-content column is active, not just signs and line numbers. The border should be present when `get_prefix_width() > 0 || prefix_column_width > 0`.

**Alternative considered**: Moving prefix column before the border (making it part of `get_prefix_width()`). Rejected because the current layout places the border between signs/line-numbers and the prefix column intentionally — the border separates "line metadata" from "entry-specific prefix + content". When all three are present, the border should remain between line numbers and the prefix column.

### Decision 2: Adjust get_offset_width to maintain correct layout

The `get_offset_width()` function is `get_prefix_width() + get_border_width() + get_custom_prefix_width()`. With the border condition fixed, when only prefix column is set:
- `get_prefix_width()` = 0
- `get_border_width()` = 1 (now correctly returns 1)
- `get_custom_prefix_width()` = prefix_column_width

This correctly places the border before the prefix column. No change to `get_offset_width()` formula is needed.

## Risks / Trade-offs

- [Risk] Existing tests assert `offset_width` and `content_width` with prefix-only viewports assuming border = 0 → Update those tests to expect border = 1
- [Risk] Visual change for users who have prefix column but no signs/line numbers — they now see a 1-cell space between icon and filename → This is the intended behavior and matches how lines with signs/line numbers already render
