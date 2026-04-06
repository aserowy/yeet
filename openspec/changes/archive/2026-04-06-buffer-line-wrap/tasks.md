## 1. Model: Add wrap option to ViewPort

- [x] 1.1 Add `wrap: bool` field to `ViewPort` in `yeet-buffer/src/model/viewport.rs`, defaulting to `false` in `Default`
- [x] 1.2 Add a `wrap_line` function to `yeet-buffer/src/model/ansi.rs` (or a new `yeet-buffer/src/view/wrap.rs` module) that takes an `Ansi` string and a `content_width` and returns a `Vec<Ansi>` of segments broken at word boundaries (last space before width), falling back to character-count breaking when no space fits. Segments consume the breaking space (it is not rendered at the end of a segment).
- [x] 1.3 Write unit tests for the wrap function: line shorter than width (1 segment), line breaks at space, word longer than width breaks by char, multiple wraps, line with ANSI codes wraps correctly, empty line returns one empty segment

## 2. View: Render wrapped lines

- [x] 2.1 Introduce a `WrapSegment` struct (or equivalent) in the view layer containing: the segment `Ansi` content, whether it is the first segment (`is_first`), the originating BufferLine index, and the character offset within the BufferLine
- [x] 2.2 Modify `get_styled_lines` in `yeet-buffer/src/view/mod.rs`: when `vp.wrap` is true, for each BufferLine, call the wrap function on `line.content` with `vp.get_content_width(&line)`, producing wrap segments. For each segment, generate the styled output — first segment gets signs/line_number/prefix/border, continuation segments get empty prefix of equal width. All segments of the cursor's BufferLine get `cursor_line_bg`.
- [x] 2.3 Modify `get_rendered_lines` in `yeet-buffer/src/view/mod.rs`: when `vp.wrap` is true, account for visual line heights — iterate from `vertical_index` and accumulate visual lines (by pre-computing wrap counts) until reaching `height`, rather than blindly taking `height` BufferLines
- [x] 2.4 Write view tests: a wrapped line produces multiple ratatui Lines, continuation lines have no line number, cursor line bg spans all segments, non-wrapped viewport still works as before

## 3. View: Cursor rendering on wrapped lines

- [x] 3.1 Modify `add_line_styles` / `add_cursor_styles` in `yeet-buffer/src/view/line.rs` to accept a `char_offset` parameter for wrap segments so the cursor position is correctly mapped within each segment. The cursor character (reverse-video block) should appear in the segment that contains the cursor's horizontal position.
- [x] 3.2 Write tests for cursor rendering: cursor on first segment, cursor on second segment, cursor at wrap boundary, cursor after `0` (BufferLine start from continuation), cursor after `$` (BufferLine end from first line), cursor after `f`/`w` crossing wrap boundary

## 4. Update: Viewport scrolling with wrap

- [x] 4.1 Modify `update_by_cursor` in `yeet-buffer/src/update/viewport.rs`: when `wrap` is true, force `horizontal_index` to 0. Calculate the visual row count of each BufferLine between `vertical_index` and the cursor to determine if the cursor's entire wrapped line is visible. Scroll so that all visual lines of the cursor's BufferLine are on screen.
- [x] 4.2 Write tests for viewport scrolling: cursor on a 3-row wrapped line near bottom scrolls to show all 3 rows, cursor moves down past wrapped lines adjusts vertical_index correctly

## 5. Frontend: Enable wrap on help buffers

- [x] 5.1 In `yeet-frontend/src/update/command/help.rs`, set `wrap: true` on the help ViewPort when creating it
- [x] 5.2 Verify help pages render with wrapping by running `cargo test` and manual inspection

## 6. Verify

- [x] 6.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
