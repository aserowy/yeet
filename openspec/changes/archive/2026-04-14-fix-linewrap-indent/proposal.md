## Why

Wrapped continuation lines are indented one space too many because the precontent border width is added twice — once inside `get_offset_width()` and once explicitly on line 125 of `yeet-buffer/src/view/mod.rs`.

## What Changes

- Fix the continuation line indentation calculation in `yeet-buffer/src/view/mod.rs` to use `get_offset_width()` alone, removing the redundant `+ get_precontent_border_width()` addition
- Add a test to verify continuation line indentation width matches the first line's prefix width

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `rendering/buffer`: Fix continuation line indentation calculation to match the documented requirement that continuation lines indent to align with the content column of the first line

## Impact

- `yeet-buffer/src/view/mod.rs` — single line fix in the wrap rendering path
