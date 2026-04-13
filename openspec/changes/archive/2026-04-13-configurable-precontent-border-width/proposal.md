## Why

The `get_precontent_border_width()` function currently always returns 1 when any pre-content column is active. This is correct for directory viewports where a visual separator between the prefix area and content is desired, but the commandline viewport also has `prefix_column_width: 1` (for command count display) and should not have a border consuming an extra cell. An optional override on `ViewPort` allows specific viewports to suppress or customize the border width.

## What Changes

- Add an optional `precontent_border_width` field (`Option<usize>`) to `ViewPort` that, when `Some(n)`, overrides the computed border width regardless of whether pre-content columns are active
- When the field is `None` (default), the existing behavior is preserved (`1` when precontent > 0, `0` otherwise)
- Set `precontent_border_width: Some(0)` on the commandline viewport to suppress the border

## Capabilities

### New Capabilities

### Modified Capabilities
- `buffer`: The `get_precontent_border_width()` function gains an optional override via `ViewPort.precontent_border_width`

## Impact

- `yeet-buffer/src/model/viewport.rs`: Add field and update `get_precontent_border_width()` logic
- `yeet-frontend/src/model/mod.rs`: Set `precontent_border_width: Some(0)` on `CommandLine::default()`
