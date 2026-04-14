## Why

Rendering image files (e.g., `assets/.face`) via the chafa fallback path produces broken output in the preview pane. Two distinct bugs are present: (1) the chafa `--view-size` width does not account for the effective content width after viewport offsets (sign columns, line numbers, prefix columns, borders), causing the rendered image to overflow or misalign when previewed in directory panes that have non-zero offsets, and (2) chafa emits non-SGR ANSI escape sequences (e.g., cursor movement codes like `\x1b[...C` or `\x1b[...H`) that the `Ansi` type's parser does not handle — it only recognizes sequences ending in `m`, so non-SGR sequences are treated as visible text, causing raw escape codes to appear in the rendered output.

## What Changes

- Adjust the rect width passed to chafa's `--view-size` to reflect the actual content width available in the preview viewport, subtracting any offset (sign columns, line numbers, prefix columns, border) so the image fits within the renderable area.
- Strip or filter non-SGR escape sequences from chafa's stdout before storing the output as `Preview::Content` lines, so that only color/style codes (ending in `m`) are preserved while cursor movement and other control sequences are removed.

## Capabilities

### New Capabilities

- `chafa-viewport-fit`: Ensure chafa image output is sized to the actual renderable content width of the preview viewport, accounting for all viewport offsets.
- `chafa-escape-sanitization`: Strip non-SGR ANSI escape sequences from chafa output so only color/style codes are passed through to the buffer rendering pipeline.

### Modified Capabilities

## Impact

- `yeet-frontend/src/task/image.rs`: The `load_with_chafa` function needs to accept or compute the effective content width rather than using the raw viewport width/height, and needs to sanitize chafa's stdout.
- `yeet-frontend/src/action.rs`: The rect construction for `Task::LoadPreview` may need to pass additional viewport context or compute the content-area dimensions.
- `yeet-buffer/src/model/ansi.rs`: May benefit from improved escape sequence handling as a defense-in-depth measure, though the primary fix should be sanitizing chafa output before it enters the buffer pipeline.
