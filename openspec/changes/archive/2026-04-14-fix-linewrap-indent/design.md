## Context

In `yeet-buffer/src/view/mod.rs`, the wrap rendering code calculates indentation for continuation lines (line 125). The current code is:

```rust
let prefix_width = vp.get_offset_width(&bl) + vp.get_precontent_border_width();
```

`get_offset_width()` already returns `get_precontent_width() + get_precontent_border_width()`. Adding `get_precontent_border_width()` again double-counts the border, producing an indentation that is 1 cell wider than the first line's prefix area.

## Goals / Non-Goals

**Goals:**
- Fix continuation line indentation to match the first line's prefix width exactly

**Non-Goals:**
- Changing how `get_offset_width` or `get_precontent_border_width` work
- Modifying wrap logic beyond the indentation calculation

## Decisions

### Use `get_offset_width()` alone for continuation indent

The continuation line indentation must equal the total prefix area width: signs + line number + prefix column + border. `get_offset_width()` already computes exactly this. Remove the redundant `+ vp.get_precontent_border_width()`.

**Before:** `vp.get_offset_width(&bl) + vp.get_precontent_border_width()`
**After:** `vp.get_offset_width(&bl)`

## Risks / Trade-offs

- [Low risk] This is a one-line change with clear semantics backed by existing unit tests for `get_offset_width`.
