# Overview

The statusline-per-window feature changes the statusline from a single global bar at the bottom of the terminal to a per-window statusline attached directly below each leaf window in the `Window` tree. When there are no splits (single window), the behavior is identical to today. When a `Horizontal` split is active, each window gets its own 1-row statusline rendered between its content and the next window (or the commandline). The focused window shows the full statusline (path, permissions, changes, position); unfocused windows show a simplified statusline (path/label only).

The implementation is split into 3 sequential prompts, each leaving the program in a compilable and functional state:

1. [Prompt 1: Move statusline rendering into the window tree traversal](#prompt-1-move-statusline-rendering-into-the-window-tree-traversal) — `planned`
2. [Prompt 2: Account for per-window statusline height in layout computation](#prompt-2-account-for-per-window-statusline-height-in-layout-computation) — `planned`
3. [Prompt 3: Differentiate focused vs unfocused statusline content](#prompt-3-differentiate-focused-vs-unfocused-statusline-content) — `planned`

---

# Prompt 1: Move statusline rendering into the window tree traversal

**Goal**: Render a statusline below each leaf window by moving statusline rendering from the top-level `view::model` into the recursive `view::buffer::render_window` traversal, so that each leaf window draws its own statusline.

**State**: `planned`

**Motivation**: Currently, a single statusline is rendered in `view::model` at a fixed y-offset after the entire window tree. This approach only shows status for the focused buffer. To support per-window statuslines in splits, the statusline must be rendered as part of each leaf window — directly below its content area. This prompt performs the structural move without changing layout math (Prompt 2) or content differentiation (Prompt 3).

## Requirements

- Each leaf window (`Window::Directory`, `Window::Tasks`) renders its own 1-row statusline immediately below its content area.
- The statusline for each leaf is rendered using the existing `statusline::view()` function, called with that leaf's buffer and viewport.
- `Window::Horizontal` does not render a statusline itself — it delegates to its children.
- Remove the statusline rendering call from `view::model`. The commandline's y-offset calculation must be updated accordingly.
- When there is a single window (no `Horizontal` split), the visual result is identical to the current behavior: one statusline row between the window content and the commandline.
- When there is a `Horizontal` split, each leaf window has its own statusline rendered below it.
- The `statusline::view()` function must become `pub(super)` or `pub(crate)` so it can be called from `view::buffer`.
- No changes to statusline content — the same information is shown as today for all windows (full content for every window in this prompt; differentiation comes in Prompt 3).

## Exclusions

- Do NOT change the layout computation in `update::window` — that is Prompt 2.
- Do NOT differentiate focused vs unfocused statusline content — that is Prompt 3.
- Do NOT change the statusline content or styling.
- Do NOT change the `Window` enum or model types.

## Context

- @yeet-frontend/src/view/mod.rs — `model()` function: currently renders statusline at a fixed y-offset after `window::view()`. This is where the statusline call must be removed.
- @yeet-frontend/src/view/buffer.rs — `view()`, `render_window()`, `render_buffer_slot()`: the recursive window tree rendering. This is where per-leaf statusline calls must be added.
- @yeet-frontend/src/view/statusline.rs — `view()`: the existing statusline rendering function. Currently takes `(buffer, viewport, frame, rect)`. Must be callable from `view::buffer`.
- @yeet-frontend/src/view/window.rs — `view()`: calls `buffer::view()` and returns the window height. Must be updated to account for statusline rows in its return value.
- @yeet-frontend/src/model/mod.rs — `Window` enum, `App`, `Contents`, `Buffer`.
- @yeet-frontend/src/update/window.rs — `update()`: layout computation (not changed in this prompt, but referenced for understanding the vertical layout constraints).
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Make `statusline::view` accessible from `view::buffer`

In `yeet-frontend/src/view/statusline.rs`, ensure the `view()` function's visibility allows it to be called from `view::buffer`. Currently it is `pub fn view(...)` which is accessible within the crate. Since `view::buffer` is a sibling module under `view`, the function is already accessible. No visibility change needed — just verify the import path works.

### Step 2: Add statusline rendering to each leaf in `render_window`

In `yeet-frontend/src/view/buffer.rs`, modify `render_window` to render a statusline after each leaf window's content. The statusline is placed at `y = viewport.y + viewport.height` (immediately below the content area), spanning the leaf's full width.

For `Window::Directory`:
```rust
Window::Directory(parent, current, preview) => {
    // ... existing render_buffer_slot calls for parent, current, preview ...

    // Render statusline below the directory panes
    let buffers = buffers; // already available
    let current_buffer = buffers.get(&current.buffer_id);
    if let Some(buffer) = current_buffer {
        let statusline_rect = Rect {
            x: 0,                      // span full width from left edge
            y: current.y + current.height, // immediately below content
            width: frame.area().width,  // full terminal width for Directory
            height: 1,
        };
        super::statusline::view(buffer, current, frame, statusline_rect);
    }
}
```

For `Window::Tasks`:
```rust
Window::Tasks(vp) => {
    render_buffer_slot(mode, frame, vp, buffers.get(&vp.buffer_id), h_off, v_off, focused_buffer_id);

    if let Some(buffer) = buffers.get(&vp.buffer_id) {
        let statusline_rect = Rect {
            x: vp.x,
            y: vp.y + vp.height,
            width: vp.width,
            height: 1,
        };
        super::statusline::view(buffer, vp, frame, statusline_rect);
    }
}
```

For `Window::Horizontal`: no statusline rendering — it delegates to its children who each render their own.

### Step 3: Remove statusline rendering from `view::model`

In `yeet-frontend/src/view/mod.rs`, remove the `statusline::view(...)` call. The commandline now renders at `y = vertical_offset` (the window height returned by `window::view`) since the statusline rows are now part of the window tree's rendered area.

Before:
```rust
let vertical_offset = window::view(model, frame).expect("Failed to render window view");
// ... statusline call ...
statusline::view(buffer, focused_vp, frame, Rect { x: 0, width: frame.area().width, y: vertical_offset, height: 1 });
commandline::view(&model.app.commandline, &model.state.modes.current, frame, vertical_offset + 1)
```

After:
```rust
let vertical_offset = window::view(model, frame).expect("Failed to render window view");
commandline::view(&model.app.commandline, &model.state.modes.current, frame, vertical_offset)
```

Note: `vertical_offset` must now account for statusline rows. This is handled by updating `window::view` to include them.

### Step 4: Update `window::view` to account for statusline height

In `yeet-frontend/src/view/window.rs`, the `view()` function returns `u16` representing the total height consumed by the window area. Since each leaf now has a 1-row statusline, the returned height must include those rows.

The simplest approach: `window::view` returns `model.app.window.get_height_with_statuslines()` — a new method on `Window` that adds 1 per leaf.

Add to `Window` impl in `yeet-frontend/src/model/mod.rs`:

```rust
/// Returns the total rendered height including per-leaf statusline rows.
pub fn get_rendered_height(&self) -> Result<u16, AppError> {
    match self {
        Window::Horizontal { first, second, .. } => {
            Ok(first.get_rendered_height()? + second.get_rendered_height()?)
        }
        Window::Directory(_, vp, _) => Ok(vp.height + 1), // +1 for statusline
        Window::Tasks(vp) => Ok(vp.height + 1),           // +1 for statusline
    }
}
```

Then in `window::view`:
```rust
pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    buffer::view(&model.state.modes.current, &model.app, frame, 0, 0);
    model.app.window.get_rendered_height()
}
```

### Step 5: Verify single-window behavior is unchanged

When there is no `Horizontal` split (just `Window::Directory`), the layout is:
- Window content: `y=0`, `height=H`
- Statusline: `y=H`, `height=1` (rendered by `render_window`)
- Commandline: `y=H+1` (rendered by `view::model`)

This matches the current behavior exactly.

### Step 6: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`

## Examples

### Single window (no split) — unchanged visual behavior
```
+-----------------------------------------------+
|  parent  |   current    |      preview         |  y=0..H-1
|          |              |                      |
+-----------------------------------------------+
| /home/user   rwxr-xr-x   +2 ~1         3/10  |  y=H (statusline)
+-----------------------------------------------+
| :command                                       |  y=H+1 (commandline)
+-----------------------------------------------+
```

### Horizontal split — each window gets its own statusline
```
+-----------------------------------------------+
|  parent  |   current    |      preview         |  y=0..H1-1 (Directory)
|          |              |                      |
+-----------------------------------------------+
| /home/user   rwxr-xr-x   +2 ~1         3/10  |  y=H1 (Directory statusline)
+-----------------------------------------------+
|  1    rg foo                                   |  y=H1+1..H1+H2 (Tasks)
|  12   fd bar                                   |
+-----------------------------------------------+
| Tasks                                    2/2   |  y=H1+H2+1 (Tasks statusline)
+-----------------------------------------------+
| :command                                       |  y=H1+H2+2 (commandline)
+-----------------------------------------------+
```

## Notes

- The statusline rect for `Window::Directory` spans the full terminal width (same as today), since the three panes (parent, current, preview) are side-by-side and the statusline sits below all of them.
- The statusline rect for `Window::Tasks` spans only the task viewport's width and x-position, in case future layouts place it beside other windows.
- After this prompt, all windows show the full statusline content (path, permissions, changes, position). Prompt 3 will differentiate focused vs unfocused.
- The `get_rendered_height()` method is separate from `get_height()` to avoid breaking existing callers of `get_height()` which is used in layout computation (where statusline height must not be double-counted).

---

# Prompt 2: Account for per-window statusline height in layout computation

**Goal**: Adjust the layout computation in `update::window` so that each leaf window reserves 1 row for its statusline, preventing the statusline from overlapping the commandline or being clipped.

**State**: `planned`

**Motivation**: After Prompt 1, each leaf window renders a statusline at `y = viewport.height`, but the layout computation in `update::window::set_buffer_vp` does not account for this extra row. The viewport heights are set to fill the available area, and the statusline ends up overlapping the commandline. This prompt fixes the layout math so that each leaf's content area is reduced by 1 row to make room for its statusline.

## Requirements

- Each leaf window's viewport height is reduced by 1 row to reserve space for its statusline.
- The global layout in `update::window::update()` no longer reserves a separate `Constraint::Length(1)` for a single statusline — that row is now accounted for within each leaf.
- `Window::Horizontal` splits allocate space for both children including their statuslines.
- When there is a single window (no split), the visual result is identical: the viewport is 1 row shorter than the allocated area, and the statusline fills that 1 row.
- The commandline y-offset is correct in all configurations (single window, horizontal split).
- Existing tests in `update::window` are updated to reflect the new viewport heights (each leaf's `height` is 1 less than before).

## Exclusions

- Do NOT change rendering code — that was done in Prompt 1.
- Do NOT change statusline content — that is Prompt 3.
- Do NOT change the `Window` enum or model types (except possibly adding a helper method).

## Context

- @yeet-frontend/src/update/window.rs — `update()`: the top-level layout splits terminal area into `[window_area, statusline(1), commandline(N)]`. The `statusline(1)` constraint must be removed. `set_buffer_vp()`: recursive layout for the window tree. Each leaf must subtract 1 from its height.
- @yeet-frontend/src/model/mod.rs — `Window::get_height()`: returns viewport height. Must remain unchanged (returns content height only, not including statusline). `Window::get_rendered_height()`: added in Prompt 1, returns content + statusline height.
- @yeet-frontend/src/view/mod.rs — `model()`: uses the returned height from `window::view()` to position the commandline. After Prompt 1, this uses `get_rendered_height()`.
- @yeet-frontend/src/view/window.rs — `view()`: returns the total rendered height.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Remove global statusline constraint from `update::window::update()`

Change the vertical layout from 3 constraints to 2:

Before:
```rust
let main = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(100),
        Constraint::Length(1),           // global statusline — remove this
        Constraint::Length(lines),       // commandline
    ])
    .split(area);

set_buffer_vp(&mut app.window, main[0])?;
set_commandline_vp(&mut app.commandline, main[2])?;
```

After:
```rust
let main = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(100),     // window area (includes per-leaf statuslines)
        Constraint::Length(lines),       // commandline
    ])
    .split(area);

set_buffer_vp(&mut app.window, main[0])?;
set_commandline_vp(&mut app.commandline, main[1])?;
```

### Step 2: Subtract 1 from each leaf's height in `set_buffer_vp`

Each leaf variant (`Window::Directory`, `Window::Tasks`) must subtract 1 from the allocated height to leave room for the statusline row:

```rust
Window::Directory(parent_vp, current_vp, preview_vp) => {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
        .split(area);

    let parent_rect = layout[0];
    let current_rect = layout[1];
    let preview_rect = layout[2];

    // Reserve 1 row for statusline
    parent_vp.height = parent_rect.height.saturating_sub(1);
    parent_vp.width = parent_rect.width;
    parent_vp.x = parent_rect.x;
    parent_vp.y = parent_rect.y;

    current_vp.height = current_rect.height.saturating_sub(1);
    current_vp.width = current_rect.width;
    current_vp.x = current_rect.x;
    current_vp.y = current_rect.y;

    preview_vp.height = preview_rect.height.saturating_sub(1);
    preview_vp.width = preview_rect.width;
    preview_vp.x = preview_rect.x;
    preview_vp.y = preview_rect.y;
}
Window::Tasks(vp) => {
    vp.height = area.height.saturating_sub(1); // Reserve 1 row for statusline
    vp.width = area.width;
    vp.x = area.x;
    vp.y = area.y;
}
```

The `saturating_sub(1)` prevents underflow if the area is somehow 0 rows tall.

### Step 3: Update existing tests

Tests in `update::window` that check viewport heights must be adjusted. For example, if a test allocates a 40-row area for a single `Window::Tasks`, the viewport height should now be 39 (40 - 1 for statusline).

Similarly, `get_height_horizontal_returns_full_area` test should verify that `get_height()` returns the content height (without statuslines), while `get_rendered_height()` returns the full area height.

### Step 4: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`

## Examples

### Layout math for single Directory window

Given terminal area height = 42, commandline lines = 1:
- Top-level split: `[Percentage(100), Length(1)]` → window area = 41 rows, commandline = 1 row
- `set_buffer_vp` for `Window::Directory`: each pane gets `height = 41 - 1 = 40` (content) + 1 (statusline rendered at y=40)
- `get_rendered_height()` = 40 + 1 = 41
- Commandline at y = 41

### Layout math for Horizontal split

Given terminal area height = 42, commandline lines = 1:
- Top-level split: window area = 41 rows, commandline = 1 row
- `set_buffer_vp` for `Window::Horizontal`: splits 41 rows into `[20, 21]` (50/50 with rounding)
- First child (Directory): pane height = 20 - 1 = 19 (content) + 1 (statusline)
- Second child (Tasks): pane height = 21 - 1 = 20 (content) + 1 (statusline)
- `get_rendered_height()` = (19 + 1) + (20 + 1) = 41
- Commandline at y = 41

## Notes

- `get_height()` continues to return only the content height (viewport height without statuslines). This method is used internally by `get_rendered_height()` and by other parts of the codebase that need the content-only height.
- `get_rendered_height()` adds 1 per leaf for the statusline. This is used by `window::view()` to tell `view::model()` where to place the commandline.
- Using `saturating_sub(1)` is a safety measure. In practice, the minimum terminal height should always provide at least 2 rows per leaf (1 content + 1 statusline).

---

# Prompt 3: Differentiate focused vs unfocused statusline content

**Goal**: Show the full statusline (path, permissions, changes, position) for the focused window and a simplified statusline (path/label only) for unfocused windows.

**State**: `planned`

**Motivation**: When multiple windows are visible, showing the full statusline for all of them creates visual clutter. Differentiating focused vs unfocused statuslines helps the user quickly identify which window is active and reduces noise. This matches the vim/neovim convention where the active window's statusline is highlighted and the inactive ones are dimmed/simplified.

## Requirements

- The focused window's statusline shows the full content: path, permissions, changes, and cursor position (for `Directory`); "Tasks" label and position (for `Tasks`). This is the existing behavior.
- Unfocused windows show a simplified statusline:
  - `Directory`: only the path (gray text on black background).
  - `Tasks`: only the "Tasks" label (gray text on black background).
- The focused statusline uses the existing `Color::Black` background.
- The unfocused statusline uses a dimmer background (e.g., `Color::DarkGray` or `Color::Rgb(30, 30, 30)`) to visually distinguish it from the focused one.
- When there is only one window (no splits), it always renders the full (focused) statusline — no visual difference from today.
- The `is_focused` boolean is passed from `render_window` into the statusline rendering call.

## Exclusions

- Do NOT change layout computation — that was done in Prompt 2.
- Do NOT change the `Window` enum or model types.
- Do NOT change any keybindings or commands.

## Context

- @yeet-frontend/src/view/statusline.rs — `view()`, `filetree_status()`, `tasks_status()`: the rendering functions. A new `is_focused` parameter will control which variant to render.
- @yeet-frontend/src/view/buffer.rs — `render_window()`: passes the focused buffer ID down through the tree. This is where `is_focused` is determined and passed to the statusline call.
- @yeet-frontend/src/model/mod.rs — `Window::focused_viewport()`: used to determine the focused buffer ID.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Add `is_focused` parameter to `statusline::view`

Change the signature of `statusline::view()` to accept a boolean:

```rust
pub fn view(current: &Buffer, viewport: &ViewPort, frame: &mut Frame, rect: Rect, is_focused: bool) {
    match current {
        Buffer::Directory(it) => {
            if is_focused {
                filetree_status(it, viewport, frame, rect)
            } else {
                filetree_status_unfocused(it, frame, rect)
            }
        }
        Buffer::Tasks(it) => {
            if is_focused {
                tasks_status(it, viewport, frame, rect)
            } else {
                tasks_status_unfocused(frame, rect)
            }
        }
        _ => {}
    }
}
```

### Step 2: Add unfocused statusline rendering functions

Add `filetree_status_unfocused` — renders only the path:

```rust
fn filetree_status_unfocused(buffer: &DirectoryBuffer, frame: &mut Frame, rect: Rect) {
    let content = buffer.path.to_str().unwrap_or("");
    let style = Style::default().fg(Color::DarkGray);
    let path = Line::from(Span::styled(content, style));

    frame.render_widget(
        Block::default().style(Style::default().bg(Color::DarkGray)),
        rect,
    );
    frame.render_widget(Paragraph::new(path), rect);
}
```

Add `tasks_status_unfocused` — renders only the "Tasks" label:

```rust
fn tasks_status_unfocused(frame: &mut Frame, rect: Rect) {
    let label = Line::from(Span::styled("Tasks", Style::default().fg(Color::Gray)));

    frame.render_widget(
        Block::default().style(Style::default().bg(Color::DarkGray)),
        rect,
    );
    frame.render_widget(Paragraph::new(label), rect);
}
```

### Step 3: Pass `is_focused` from `render_window`

In `yeet-frontend/src/view/buffer.rs`, the `render_window` function already has `focused_buffer_id`. Use it to determine if the current leaf is focused:

For `Window::Directory`:
```rust
let is_focused = current.buffer_id == focused_buffer_id;
super::statusline::view(buffer, current, frame, statusline_rect, is_focused);
```

For `Window::Tasks`:
```rust
let is_focused = vp.buffer_id == focused_buffer_id;
super::statusline::view(buffer, vp, frame, statusline_rect, is_focused);
```

### Step 4: Choose unfocused background color

Use `Color::DarkGray` as the background for unfocused statuslines. This provides a visible but subtle distinction from the focused `Color::Black` background. The unfocused text uses `Color::Gray` to further dim it.

Alternative: Use `Color::Rgb(40, 40, 40)` for a more subtle distinction. Choose based on visual testing.

### Step 5: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`

## Examples

### Focused Directory statusline (unchanged from today)
```
| /home/user   rwxr-xr-x   +2 ~1         3/10  |  (black bg, gray text)
```

### Unfocused Directory statusline (simplified)
```
| /home/user                                     |  (dark gray bg, dim text)
```

### Focused Tasks statusline (unchanged from today)
```
| Tasks                                    2/2   |  (black bg, gray text)
```

### Unfocused Tasks statusline (simplified)
```
| Tasks                                          |  (dark gray bg, dim text)
```

### Full split layout with focus on Directory
```
+-----------------------------------------------+
|  parent  |   current    |      preview         |  (Directory content)
+-----------------------------------------------+
| /home/user   rwxr-xr-x   +2 ~1         3/10  |  <- FOCUSED (black bg, full info)
+-----------------------------------------------+
|  1    rg foo                                   |  (Tasks content)
+-----------------------------------------------+
| Tasks                                          |  <- UNFOCUSED (dark gray bg, label only)
+-----------------------------------------------+
| :command                                       |
+-----------------------------------------------+
```

### Full split layout with focus on Tasks
```
+-----------------------------------------------+
|  parent  |   current    |      preview         |  (Directory content)
+-----------------------------------------------+
| /home/user                                     |  <- UNFOCUSED (dark gray bg, path only)
+-----------------------------------------------+
|  1    rg foo                                   |  (Tasks content)
+-----------------------------------------------+
| Tasks                                    2/2   |  <- FOCUSED (black bg, full info)
+-----------------------------------------------+
| :command                                       |
+-----------------------------------------------+
```

## Notes

- The `Color::DarkGray` background is a terminal color that may render differently across terminals. If visual testing shows poor contrast, consider using `Color::Rgb(40, 40, 40)` for more control.
- The unfocused statusline functions are intentionally separate from the focused ones (not parameterized) to keep each rendering path simple and easy to modify independently.
- When there is no split, the single window is always "focused" so `is_focused` is always `true` — the full statusline is shown, identical to today's behavior.
