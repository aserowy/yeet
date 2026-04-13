## Why

When `prefix_column_width > 0` but `sign_column_width == 0` and `line_number == None`, the border (single space separator) between pre-content columns and the content area is not rendered. The `get_border_width()` function only checks `get_prefix_width()` (signs + line numbers), ignoring the prefix column. This means directory entries with icons but no signs/line numbers have their icon glued directly to the filename with no visual separation.

## What Changes

- Fix `get_border_width()` in `yeet-buffer/src/model/viewport.rs` to account for `prefix_column_width` when deciding whether a border space is needed
- Update `get_offset_width()` layout calculation so the border space is correctly positioned between the prefix column and content when only the prefix column is active

## Capabilities

### New Capabilities

### Modified Capabilities
- `buffer`: The border rendering condition changes to include prefix column width, ensuring the 1-cell border space is always present when any pre-content column is active

## Impact

- `yeet-buffer/src/model/viewport.rs`: `get_border_width()` and potentially `get_offset_width()` logic
- Existing tests in `viewport.rs` that assert offset/content widths when only prefix column is set will need updating (border width changes from 0 to 1)
- Visual change: lines with prefix column but no signs/line numbers will now have a 1-cell space between the icon and content
