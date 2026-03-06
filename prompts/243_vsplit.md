# Overview

The vsplit feature adds vertical splitting (side-by-side) to yeet's window tree. Currently, the `Window` enum only supports `Horizontal` splits (top/bottom). This feature introduces a `Vertical` split variant that divides the available area into left and right halves, along with correct rendering, layout computation, per-window statuslines, and focus movement in all four directions through a mixed tree of `Horizontal` and `Vertical` splits.

This prompt file does **not** cover keymaps and commands to create vertical and horizontal splits — those are implemented separately. It only covers the infrastructure: the model types, layout, rendering, and focus navigation.

The implementation is split into 4 sequential prompts, each leaving the program in a compilable and functional state:

1. [Prompt 1: Add `Window::Vertical` variant to the model](#prompt-1-add-windowvertical-variant-to-the-model) — `done`
2. [Prompt 2: Implement `Window::Vertical` in layout computation and all `Window` match sites](#prompt-2-implement-windowvertical-in-layout-computation-and-all-window-match-sites) — `done`
3. [Prompt 3: Render `Window::Vertical` splits with per-window statuslines](#prompt-3-render-windowvertical-splits-with-per-window-statuslines) — `done`
4. [Prompt 4: Extend focus navigation to support all four directions across mixed split trees](#prompt-4-extend-focus-navigation-to-support-all-four-directions-across-mixed-split-trees) — `planned`

---

# Prompt 1: Add `Window::Vertical` variant to the model

**Goal**: Introduce the `Window::Vertical` variant as a new internal node in the window tree, with the same structure as `Window::Horizontal` but representing a side-by-side (left/right) split. Add `todo!()` arms to all exhaustive matches on `Window`.

**State**: `done`

**Motivation**: The window tree currently only supports top/bottom splits (`Horizontal`). To enable side-by-side splits, the `Window` enum needs a `Vertical` variant. Adding the type first — with all match arms compiling via `todo!()` — isolates the model change from logic changes, keeping the program functional and each step reviewable.

## Requirements

- Add `Window::Vertical { first: Box<Window>, second: Box<Window>, focus: SplitFocus }` to the `Window` enum. Semantically, `first` is the left pane and `second` is the right pane.
- Add `Window::Vertical { .. }` arms with `todo!()` to every exhaustive match on `Window` across the entire codebase.
- Reuse the existing `SplitFocus` enum — no new types needed. `SplitFocus::First` means the left pane is focused, `SplitFocus::Second` means the right pane is focused.
- All existing tests continue to pass.
- Add a test that constructs a `Window::Vertical` tree and pattern-matches it.

## Exclusions

- Do NOT implement any `Vertical` logic (leave `todo!()`).
- Do NOT change layout computation, rendering, or focus switching.
- Do NOT change runtime behavior — this is a types-only change.

## Context

- @yeet-frontend/src/model/mod.rs — `Window` enum (line ~85), `SplitFocus`, all `Window` methods (`get_height`, `focused_viewport`, `focused_viewport_mut`, `buffer_ids`, `contains_tasks`).
- @yeet-frontend/src/update/app.rs — `get_focused_directory_viewports`, `get_focused_directory_viewports_mut`, `get_viewport_by_buffer_id_mut`.
- @yeet-frontend/src/update/window.rs — `set_buffer_vp`.
- @yeet-frontend/src/update/focus.rs — `change`.
- @yeet-frontend/src/update/buffers.rs — `update`.
- @yeet-frontend/src/update/command/mod.rs — `close_focused_window_or_quit`, `reset_unsaved_changes`.
- @yeet-frontend/src/view/buffer.rs — `render_window`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Add `Window::Vertical` variant** in `yeet-frontend/src/model/mod.rs`:

   ```rust
   #[allow(clippy::large_enum_variant)]
   pub enum Window {
       Horizontal {
           first: Box<Window>,
           second: Box<Window>,
           focus: SplitFocus,
       },
       Vertical {
           first: Box<Window>,
           second: Box<Window>,
           focus: SplitFocus,
       },
       Directory(ViewPort, ViewPort, ViewPort),
       Tasks(ViewPort),
   }
   ```

2. **Add `Window::Vertical { .. }` arms with `todo!()`** to every exhaustive match on `Window`. Find all sites by searching for `Window::Horizontal` — every match that handles `Horizontal` must also handle `Vertical`. Known sites:
   - `yeet-frontend/src/model/mod.rs` — `get_height()`, `focused_viewport()`, `focused_viewport_mut()`, `buffer_ids()`, `contains_tasks()`, `Default` impl (no change needed for `Default`).
   - `yeet-frontend/src/update/app.rs` — `get_focused_directory_viewports`, `get_focused_directory_viewports_mut`, `get_viewport_by_buffer_id_mut`.
   - `yeet-frontend/src/update/window.rs` — `set_buffer_vp`.
   - `yeet-frontend/src/update/focus.rs` — `change`.
   - `yeet-frontend/src/update/command/mod.rs` — `close_focused_window_or_quit`, `reset_unsaved_changes`.
   - `yeet-frontend/src/view/buffer.rs` — `render_window`.

3. **Add tests** in the existing `mod test` block of `yeet-frontend/src/model/mod.rs`:

   ```rust
   #[test]
   fn window_vertical_construction_and_pattern_match() {
       let tree = Window::Vertical {
           first: Box::new(Window::Directory(
               ViewPort::default(),
               ViewPort::default(),
               ViewPort::default(),
           )),
           second: Box::new(Window::Directory(
               ViewPort::default(),
               ViewPort::default(),
               ViewPort::default(),
           )),
           focus: SplitFocus::First,
       };
       assert!(matches!(tree, Window::Vertical { .. }));
   }
   ```

4. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// Window::Vertical construction
let tree = Window::Vertical {
    first: Box::new(Window::Directory(
        ViewPort::default(), ViewPort::default(), ViewPort::default(),
    )),
    second: Box::new(Window::Directory(
        ViewPort::default(), ViewPort::default(), ViewPort::default(),
    )),
    focus: SplitFocus::First,
};
assert!(matches!(tree, Window::Vertical { .. }));

// Nested: Horizontal containing a Vertical
let nested = Window::Horizontal {
    first: Box::new(tree),
    second: Box::new(Window::Tasks(ViewPort::default())),
    focus: SplitFocus::First,
};
assert!(matches!(nested, Window::Horizontal { .. }));
```

## Notes

- The `Vertical` variant is structurally identical to `Horizontal` — both are binary splits with a `SplitFocus`. The semantic difference is the split direction: `Horizontal` splits top/bottom, `Vertical` splits left/right.
- The `todo!()` bodies will be replaced in Prompt 2. They are safe for now because no code path currently constructs a `Window::Vertical`.
- Search for `Window::Horizontal` across the codebase to find all exhaustive match sites. The Rust compiler will also report `non-exhaustive patterns` errors if any are missed.

---

# Prompt 2: Implement `Window::Vertical` in layout computation and all `Window` match sites

**Goal**: Replace every `todo!()` on `Window::Vertical` with real implementations, making the window tree fully functional for vertical (left/right) splits. The `Vertical` variant behaves like `Horizontal` but splits the area horizontally (left/right, 50/50) instead of vertically (top/bottom).

**State**: `done`

**Motivation**: The `Window::Vertical` variant was added in Prompt 1 with `todo!()` arms. Before any user-facing feature can create vertical splits, all infrastructure — layout computation, focus-aware helpers, buffer-id collection, and window closing — must handle `Vertical` correctly.

## Requirements

- Replace all `todo!()` sites for `Window::Vertical` with working logic.
- `Vertical` layout splits the area horizontally (left/right, 50/50) using `Direction::Horizontal` in ratatui's `Layout`.
- Focus-aware functions (`focused_viewport`, `focused_viewport_mut`, `get_focused_directory_viewports`, etc.) recurse into the focused child of `Vertical`, exactly like `Horizontal`.
- `buffer_ids()` recursively walks both children of `Vertical`.
- `contains_tasks()` recursively checks both children of `Vertical`.
- `get_height()` for `Vertical` returns the **maximum** of its two children's heights (since they are side-by-side, the taller child determines the split's height).
- `get_viewport_by_buffer_id_mut` searches both children of `Vertical`.
- `close_focused_window_or_quit` handles `Vertical` the same way as `Horizontal` — close the focused child, keep the other.
- `set_buffer_vp` splits the area with `Direction::Horizontal` + `Constraint::from_ratios([(1, 2), (1, 2)])` for `Vertical`.
- All existing tests continue to pass. New tests verify `Vertical`-specific behavior.

## Exclusions

- Do NOT add commands or keybindings to create vertical splits.
- Do NOT change rendering — that is Prompt 3.
- Do NOT change focus navigation logic — that is Prompt 4.
- This prompt only makes the window tree infrastructure work for `Vertical`.

## Context

- @yeet-frontend/src/model/mod.rs — `Window` enum, `SplitFocus`, `get_height()`, `focused_viewport()`, `focused_viewport_mut()`, `buffer_ids()`, `contains_tasks()`.
- @yeet-frontend/src/update/app.rs — `get_focused_directory_viewports`, `get_focused_directory_viewports_mut`, `get_focused_directory_buffer_ids`, `get_viewport_by_buffer_id_mut`.
- @yeet-frontend/src/update/window.rs — `set_buffer_vp`, uses `ratatui::layout::{Layout, Direction, Constraint, Rect}`.
- @yeet-frontend/src/update/focus.rs — `change` (leave focus logic as-is for now — just ensure `Vertical` doesn't panic; Prompt 4 will implement proper focus).
- @yeet-frontend/src/update/command/mod.rs — `close_focused_window_or_quit`, `reset_unsaved_changes`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Implement `Window` methods for `Vertical` in `yeet-frontend/src/model/mod.rs`

For each method, the `Vertical` arm mirrors the `Horizontal` arm except where noted:

**`get_height()`** — returns the **max** of both children (side-by-side layout):
```rust
Window::Vertical { first, second, .. } => {
    Ok(first.get_height()?.max(second.get_height()?))
}
```

**`focused_viewport()` / `focused_viewport_mut()`** — same recursion pattern as `Horizontal`:
```rust
Window::Vertical { first, second, focus } => match focus {
    SplitFocus::First => first.focused_viewport(),
    SplitFocus::Second => second.focused_viewport(),
},
```

**`buffer_ids()`** — union of both children, same as `Horizontal`:
```rust
Window::Vertical { first, second, .. } => {
    let mut ids = first.buffer_ids();
    ids.extend(second.buffer_ids());
    ids
}
```

**`contains_tasks()`** — same as `Horizontal`:
```rust
Window::Vertical { first, second, .. } => {
    first.contains_tasks() || second.contains_tasks()
}
```

### Step 2: Implement `Vertical` in `update/app.rs` helpers

**`get_focused_directory_viewports` / `_mut`** — same recursion pattern as `Horizontal`:
```rust
Window::Vertical { first, second, focus } => match focus {
    SplitFocus::First => get_focused_directory_viewports(first),
    SplitFocus::Second => get_focused_directory_viewports(second),
},
```

**`get_viewport_by_buffer_id_mut`** — same as `Horizontal`:
```rust
Window::Vertical { first, second, .. } => {
    get_viewport_by_buffer_id_mut(first, buffer_id)
        .or_else(|| get_viewport_by_buffer_id_mut(second, buffer_id))
}
```

### Step 3: Implement `Vertical` layout in `update/window.rs`

In `set_buffer_vp`, add the `Vertical` arm that splits the area left/right 50/50:

```rust
Window::Vertical { first, second, .. } => {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_ratios([(1, 2), (1, 2)]))
        .split(area);
    set_buffer_vp(first, layout[0])?;
    set_buffer_vp(second, layout[1])?;
}
```

### Step 4: Handle `Vertical` in `update/focus.rs`

For now, mirror the `Horizontal` handling but swap directions: `Left` → `First`, `Right` → `Second`, `Up`/`Down` → no-op. This is a minimal placeholder — Prompt 4 will replace this with proper tree-walking focus navigation.

```rust
Window::Vertical { first, second, focus } => {
    let new_focus = match direction {
        FocusDirection::Left => SplitFocus::First,
        FocusDirection::Right => SplitFocus::Second,
        FocusDirection::Up | FocusDirection::Down => return Vec::new(),
    };
    // ... same cursor toggle and focus update as Horizontal
}
```

Note: Since the current `change()` function early-returns for non-`Horizontal` roots, you need to restructure it to also match `Vertical` at the root level. Extract the cursor-toggle + focus-update logic into a shared helper to avoid duplication:

```rust
pub fn change(app: &mut App, direction: &FocusDirection) -> Vec<Action> {
    match &mut app.window {
        Window::Horizontal { first, second, focus } => {
            let new_focus = match direction {
                FocusDirection::Down => SplitFocus::Second,
                FocusDirection::Up => SplitFocus::First,
                FocusDirection::Left | FocusDirection::Right => return Vec::new(),
            };
            update_focus(first, second, focus, new_focus)
        }
        Window::Vertical { first, second, focus } => {
            let new_focus = match direction {
                FocusDirection::Right => SplitFocus::Second,
                FocusDirection::Left => SplitFocus::First,
                FocusDirection::Up | FocusDirection::Down => return Vec::new(),
            };
            update_focus(first, second, focus, new_focus)
        }
        _ => Vec::new(),
    }
}

fn update_focus(
    first: &mut Window,
    second: &mut Window,
    focus: &mut SplitFocus,
    new_focus: SplitFocus,
) -> Vec<Action> {
    if *focus == new_focus {
        return Vec::new();
    }
    match focus {
        SplitFocus::First => {
            first.focused_viewport_mut().hide_cursor = true;
            second.focused_viewport_mut().hide_cursor = false;
        }
        SplitFocus::Second => {
            second.focused_viewport_mut().hide_cursor = true;
            first.focused_viewport_mut().hide_cursor = false;
        }
    }
    *focus = new_focus;
    Vec::new()
}
```

### Step 5: Handle `Vertical` in `update/command/mod.rs`

**`close_focused_window_or_quit`** — handle `Vertical` the same as `Horizontal`:
```rust
Window::Vertical { first, second, focus } => {
    let (kept, dropped) = match focus {
        SplitFocus::First => (*second, *first),
        SplitFocus::Second => (*first, *second),
    };
    if discard_changes {
        reset_unsaved_changes(&dropped, &mut app.contents);
    }
    app.window = kept;
    add_change_mode(mode_before, Mode::Navigation, Vec::new())
}
```

**`reset_unsaved_changes`** — uses `window.buffer_ids()` which already handles `Vertical` from Step 1. No changes needed.

### Step 6: Add tests

In `yeet-frontend/src/model/mod.rs` tests:
```rust
#[test]
fn get_height_vertical_returns_max_of_children() {
    let tree = Window::Vertical {
        first: Box::new(Window::Tasks(ViewPort { height: 10, ..Default::default() })),
        second: Box::new(Window::Tasks(ViewPort { height: 15, ..Default::default() })),
        focus: SplitFocus::First,
    };
    // height = max(10+1, 15+1) = 16
    assert_eq!(tree.get_height().unwrap(), 16);
}

#[test]
fn focused_viewport_follows_vertical_split_focus() {
    let tree = Window::Vertical {
        first: Box::new(Window::Tasks(ViewPort { height: 10, ..Default::default() })),
        second: Box::new(Window::Tasks(ViewPort { height: 20, ..Default::default() })),
        focus: SplitFocus::Second,
    };
    assert_eq!(tree.focused_viewport().height, 20);
}

#[test]
fn buffer_ids_collects_from_vertical() {
    let tree = Window::Vertical {
        first: Box::new(Window::Tasks(ViewPort { buffer_id: 1, ..Default::default() })),
        second: Box::new(Window::Tasks(ViewPort { buffer_id: 2, ..Default::default() })),
        focus: SplitFocus::First,
    };
    let ids = tree.buffer_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
}
```

In `yeet-frontend/src/update/window.rs` tests:
```rust
#[test]
fn set_buffer_vp_vertical_splits_horizontally() {
    let mut tree = Window::Vertical {
        first: Box::new(Window::Tasks(ViewPort::default())),
        second: Box::new(Window::Tasks(ViewPort::default())),
        focus: SplitFocus::First,
    };
    let area = Rect { x: 0, y: 0, width: 80, height: 40 };
    set_buffer_vp(&mut tree, area).unwrap();

    match &tree {
        Window::Vertical { first, second, .. } => {
            match (first.as_ref(), second.as_ref()) {
                (Window::Tasks(left), Window::Tasks(right)) => {
                    assert_eq!(left.x, 0);
                    assert!(right.x > 0, "right pane x should be > 0");
                    assert_eq!(left.y, 0);
                    assert_eq!(right.y, 0);
                    assert!(left.width > 0);
                    assert!(right.width > 0);
                    assert_eq!(left.width + right.width, 80);
                }
                _ => panic!("expected Tasks"),
            }
        }
        _ => panic!("expected Vertical"),
    }
}
```

In `yeet-frontend/src/update/focus.rs` tests:
```rust
#[test]
fn right_moves_focus_on_vertical() {
    let mut app = make_vertical_app();
    change(&mut app, &FocusDirection::Right);
    match &app.window {
        Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::Second),
        _ => panic!("expected Vertical"),
    }
}

#[test]
fn up_down_noop_on_vertical() {
    let mut app = make_vertical_app();
    change(&mut app, &FocusDirection::Up);
    match &app.window {
        Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::First),
        _ => panic!("expected Vertical"),
    }
    change(&mut app, &FocusDirection::Down);
    match &app.window {
        Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::First),
        _ => panic!("expected Vertical"),
    }
}
```

### Step 7: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

```rust
// Layout: Vertical split with two Tasks windows
// Given area { x: 0, y: 0, width: 80, height: 40 }:
// first (left)  gets { x: 0,  y: 0, width: 40, height: 40 }
// second (right) gets { x: 40, y: 0, width: 40, height: 40 }

// Nested: Vertical containing two Directory windows
// Each Directory gets half the width, then internally splits into 3 columns
// Left Directory:  parent(1/5 of 40=8), current(2/5 of 40=16), preview(2/5 of 40=16)
// Right Directory: parent(1/5 of 40=8), current(2/5 of 40=16), preview(2/5 of 40=16)
```

## Notes

- `get_height()` uses `max()` for `Vertical` because side-by-side children share the same vertical space. The split's total height is determined by the taller child. This differs from `Horizontal` which uses `+` because its children are stacked vertically.
- The focus handling in Step 4 is a simple placeholder. It only works for root-level `Vertical` splits. Prompt 4 will implement proper tree-walking focus that handles any nesting of `Horizontal` and `Vertical`.
- `close_focused_window_or_quit` is structurally identical for `Horizontal` and `Vertical`. Consider extracting a shared helper if the duplication becomes unwieldy — but for now, matching both variants separately is fine for clarity.

---

# Prompt 3: Render `Window::Vertical` splits with per-window statuslines

**Goal**: Make vertical splits visible by implementing the `Window::Vertical` arm in the rendering pipeline, including per-window statuslines for each leaf in a vertical split.

**State**: `done`

**Motivation**: After Prompt 2, vertical splits are fully functional in the model and layout, but nothing renders. The `render_window` function in `view/buffer.rs` has a `todo!()` (or no arm yet) for `Window::Vertical`. This prompt connects the model to the view so the user sees side-by-side windows.

## Requirements

- `render_window` handles `Window::Vertical` by recursively rendering both children, same pattern as `Horizontal`.
- Each leaf window in a vertical split renders its own per-window statusline below its content area.
- The statusline for a leaf in a `Vertical` split spans only that leaf's width (not the full terminal width), since the leaves are side-by-side.
- Cursor visibility for focused vs unfocused windows is handled at render time, same as existing `Horizontal` behavior — no model state is mutated.
- The focused/unfocused statusline differentiation (full content vs simplified) works correctly for leaves within `Vertical` splits.
- **Special case for `Window::Directory` inside a `Vertical` split**: The `Directory` statusline currently spans `frame.area().width` (full terminal width). When a `Directory` is inside a `Vertical` split, it must span only the `Directory`'s allocated width. Determine the correct width from the viewports' layout rather than from `frame.area()`.

## Exclusions

- Do NOT change layout computation — that was done in Prompt 2.
- Do NOT change focus navigation — that is Prompt 4.
- Do NOT add commands or keybindings.

## Context

- @yeet-frontend/src/view/buffer.rs — `render_window`, `render_buffer_slot`: the recursive rendering functions. `render_window` needs a `Vertical` arm. The `Directory` statusline width must be computed from the pane layout, not `frame.area().width`.
- @yeet-frontend/src/view/statusline.rs — `view()`, `filetree_status`, `tasks_status`: per-window statusline rendering.
- @yeet-frontend/src/update/window.rs — `set_buffer_vp`: reference for how `Vertical` layout is computed (from Prompt 2).
- @yeet-frontend/src/model/mod.rs — `Window` enum, `ViewPort`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Add `Window::Vertical` arm to `render_window` in `view/buffer.rs`

The `Vertical` arm is identical to `Horizontal` — recursively render both children:

```rust
Window::Vertical { first, second, .. } => {
    render_window(mode, first, buffers, frame, horizontal_offset, vertical_offset, focused_buffer_id);
    render_window(mode, second, buffers, frame, horizontal_offset, vertical_offset, focused_buffer_id);
}
```

### Step 2: Fix `Directory` statusline width for splits

Currently, the `Directory` arm in `render_window` computes the statusline rect with `width: frame.area().width`. This is correct when the `Directory` occupies the full terminal width, but inside a `Vertical` split it only occupies half the width.

Change the statusline width to be computed from the viewports' positions instead of `frame.area()`:

```rust
Window::Directory(parent, current, preview) => {
    // ... render_buffer_slot calls ...

    if let Some(buffer) = buffers.get(&current.buffer_id) {
        let is_focused = current.buffer_id == focused_buffer_id;
        // Compute the total width from the leftmost to rightmost pane
        let total_width = (preview.x + preview.width) - parent.x;
        let statusline_rect = Rect {
            x: parent.x.saturating_add(horizontal_offset),
            y: current
                .y
                .saturating_add(current.height)
                .saturating_add(vertical_offset),
            width: total_width,
            height: 1,
        };
        statusline::view(buffer, current, frame, statusline_rect, is_focused);
    }
}
```

This change is safe for non-split `Directory` windows too: when there is no split, `parent.x` is 0 and `preview.x + preview.width` equals the full terminal width, so the behavior is identical.

### Step 3: Verify `Tasks` statusline width

The `Tasks` arm already uses `vp.width` for the statusline rect width, which is correct — it automatically adapts to the viewport's allocated width regardless of splits. No changes needed.

### Step 4: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

### Vertical split: two Directory windows side-by-side
```
+-------------------+-------------------+
| par | curr | prev | par | curr | prev |  <- Two Directory windows (left, right)
|     |      |      |     |      |      |
+-------------------+-------------------+
| /home/user  3/10  | /tmp       5/20   |  <- Per-window statuslines (each spans half width)
+-------------------+-------------------+
| :command                               |  <- Commandline (full width)
+---------------------------------------+
```

### Vertical split: Directory left, Tasks right
```
+-------------------+-------------------+
| par | curr | prev |  1    rg foo      |
|     |      |      |  12   fd bar      |
+-------------------+-------------------+
| /home/user  3/10  | Tasks        2/2  |  <- Per-window statuslines
+-------------------+-------------------+
| :command                               |
+---------------------------------------+
```

### Mixed: Horizontal containing a Vertical (left) and Tasks (bottom)
```
+-------------------+-------------------+
| par | curr | prev | par | curr | prev |  <- Vertical split (top half)
+-------------------+-------------------+
| /home/user  3/10  | /tmp       5/20   |  <- Per-window statuslines for Vertical children
+-------------------+-------------------+
|  1    rg foo                           |  <- Tasks (bottom half)
+---------------------------------------+
| Tasks                            2/2   |  <- Tasks statusline
+---------------------------------------+
| :command                               |
+---------------------------------------+
```

## Notes

- The key rendering change is computing the `Directory` statusline width from the viewports' layout rather than `frame.area().width`. This makes statuslines adapt automatically to any split configuration.
- The `Vertical` rendering arm is trivially identical to `Horizontal` — just recurse into both children. The layout positions were already set correctly by `set_buffer_vp` in Prompt 2.
- Cursor visibility is already handled by `render_buffer_slot` which compares each viewport's `buffer_id` against the focused `buffer_id`. No changes needed for `Vertical` splits.

---

# Prompt 4: Extend focus navigation to support all four directions across mixed split trees

**Goal**: Replace the current single-level focus switching with a tree-walking algorithm that correctly moves focus in all four directions (`Up`, `Down`, `Left`, `Right`) through any nesting of `Horizontal` and `Vertical` splits.

**State**: `planned`

**Motivation**: The current `focus::change` function only handles one level of `Horizontal` or `Vertical` at the root. In a nested tree (e.g., a `Vertical` inside a `Horizontal`), pressing `Ctrl+j`/`Ctrl+k` should move focus between windows that are visually above/below each other, even if they are in different branches of the tree. Similarly, `Ctrl+h`/`Ctrl+l` should move between windows that are visually left/right. This requires walking the tree to find the correct sibling in the desired direction.

## Requirements

- Focus movement works correctly for any nesting depth of `Horizontal` and `Vertical` splits.
- `Up`/`Down` moves focus between windows that are vertically adjacent. In a `Horizontal` split, this moves between `first` (top) and `second` (bottom). In a `Vertical` split, this propagates through to find a `Horizontal` ancestor or is a no-op.
- `Left`/`Right` moves focus between windows that are horizontally adjacent. In a `Vertical` split, this moves between `first` (left) and `second` (right). In a `Horizontal` split, this propagates through to find a `Vertical` ancestor or is a no-op.
- Cursor visibility toggles correctly when focus changes (hide on old leaf, show on new leaf).
- When moving focus into a branch (e.g., from a leaf into a subtree), focus enters the **nearest** leaf in the direction of movement. For example, pressing `Right` into a `Horizontal` subtree focuses the left-most leaf of that subtree (the `first` child, recursively).
- All existing focus tests continue to pass (adapted to new function signatures if needed).

## Exclusions

- Do NOT add commands or keybindings to create splits.
- Do NOT change layout computation or rendering.
- Do NOT change any types in the model.

## Context

- @yeet-frontend/src/update/focus.rs — current `change` function that handles single-level focus switching. This will be rewritten.
- @yeet-frontend/src/model/mod.rs — `Window` enum, `SplitFocus`, `focused_viewport()`, `focused_viewport_mut()`.
- @yeet-keymap/src/message.rs — `FocusDirection { Up, Down, Left, Right }`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Design the tree-walking algorithm

The algorithm works by walking **up** the tree from the currently focused leaf until it finds a split node where the requested direction is meaningful, then walking **down** into the target subtree to find the nearest leaf.

**Direction-to-split mapping:**
- `Up`/`Down` are meaningful at `Horizontal` splits (top/bottom children).
- `Left`/`Right` are meaningful at `Vertical` splits (left/right children).

**Algorithm outline:**

1. Find the path from the root to the currently focused leaf (a list of `(node, which_child_was_taken)` entries).
2. Walk backwards along this path looking for a split node where:
   - The direction is relevant to the split type (`Up`/`Down` for `Horizontal`, `Left`/`Right` for `Vertical`).
   - The direction points to the **other** child (i.e., the focus is currently on one side, and the direction points to the opposite side).
3. If found, change `focus` on that split node, then walk into the newly focused subtree to find the nearest leaf (entering from the appropriate side).
4. If no such ancestor is found, the direction is a no-op.

**Entering a subtree from a direction:**
When focus moves into a subtree, choose which child to focus based on the direction:
- Entering from `Up` (i.e., moving down into a subtree) → focus the `First` child of `Horizontal` splits, and the currently focused child of `Vertical` splits.
- Entering from `Down` (i.e., moving up into a subtree) → focus the `Second` child of `Horizontal` splits, and the currently focused child of `Vertical` splits.
- Entering from `Left` (i.e., moving right into a subtree) → focus the `First` child of `Vertical` splits, and the currently focused child of `Horizontal` splits.
- Entering from `Right` (i.e., moving left into a subtree) → focus the `Second` child of `Vertical` splits, and the currently focused child of `Horizontal` splits.

### Step 2: Implement the algorithm

The approach uses a recursive `try_move` function that returns `true` if focus was changed, `false` otherwise. The function tries to move within the current node's children first, then the caller (parent) handles propagation.

```rust
pub fn change(app: &mut App, direction: &FocusDirection) -> Vec<Action> {
    let old_focused_id = app.window.focused_viewport().buffer_id;
    let changed = try_move(&mut app.window, direction);
    if changed {
        let new_focused_id = app.window.focused_viewport().buffer_id;
        if old_focused_id != new_focused_id {
            // Toggle cursor visibility — handled via render-time in view/buffer.rs,
            // but we also update the model for focus::change cursor hide/show pattern.
            // Find and hide cursor on old, show on new.
            if let Some(old_vp) = crate::update::app::get_viewport_by_buffer_id_mut(&mut app.window, old_focused_id) {
                old_vp.hide_cursor = true;
            }
            app.window.focused_viewport_mut().hide_cursor = false;
        }
    }
    Vec::new()
}

/// Attempts to move focus in `direction` within or below `window`.
/// Returns `true` if focus was successfully changed.
fn try_move(window: &mut Window, direction: &FocusDirection) -> bool {
    match window {
        Window::Horizontal { first, second, focus } => {
            // First, try to move within the currently focused child
            let moved = match focus {
                SplitFocus::First => try_move(first, direction),
                SplitFocus::Second => try_move(second, direction),
            };
            if moved {
                return true;
            }
            // If the child couldn't handle it, try at this level
            match direction {
                FocusDirection::Down if *focus == SplitFocus::First => {
                    *focus = SplitFocus::Second;
                    enter_from(second, direction);
                    true
                }
                FocusDirection::Up if *focus == SplitFocus::Second => {
                    *focus = SplitFocus::First;
                    enter_from(first, direction);
                    true
                }
                _ => false,
            }
        }
        Window::Vertical { first, second, focus } => {
            let moved = match focus {
                SplitFocus::First => try_move(first, direction),
                SplitFocus::Second => try_move(second, direction),
            };
            if moved {
                return true;
            }
            match direction {
                FocusDirection::Right if *focus == SplitFocus::First => {
                    *focus = SplitFocus::Second;
                    enter_from(second, direction);
                    true
                }
                FocusDirection::Left if *focus == SplitFocus::Second => {
                    *focus = SplitFocus::First;
                    enter_from(first, direction);
                    true
                }
                _ => false,
            }
        }
        // Leaf nodes can never handle focus movement themselves
        Window::Directory(_, _, _) | Window::Tasks(_) => false,
    }
}

/// When focus enters a subtree from a given direction, recursively set focus
/// to the nearest leaf on the entry side.
fn enter_from(window: &mut Window, direction: &FocusDirection) {
    match window {
        Window::Horizontal { first, second, focus } => {
            match direction {
                // Moving down → enter from top → focus first (top)
                FocusDirection::Down | FocusDirection::Right => {
                    *focus = SplitFocus::First;
                    enter_from(first, direction);
                }
                // Moving up → enter from bottom → focus second (bottom)
                FocusDirection::Up | FocusDirection::Left => {
                    *focus = SplitFocus::Second;
                    enter_from(second, direction);
                }
            }
        }
        Window::Vertical { first, second, focus } => {
            match direction {
                // Moving right → enter from left → focus first (left)
                FocusDirection::Right | FocusDirection::Down => {
                    *focus = SplitFocus::First;
                    enter_from(first, direction);
                }
                // Moving left → enter from right → focus second (right)
                FocusDirection::Left | FocusDirection::Up => {
                    *focus = SplitFocus::Second;
                    enter_from(second, direction);
                }
            }
        }
        Window::Directory(_, _, _) | Window::Tasks(_) => {
            // Leaf — nothing to recurse into
        }
    }
}
```

### Step 3: Update cursor visibility management

The cursor hide/show logic in the current `change()` function manually toggles `hide_cursor` on the old and new focused viewports. The new implementation should do the same, but after the tree-walking is complete:

```rust
pub fn change(app: &mut App, direction: &FocusDirection) -> Vec<Action> {
    let old_focused_id = app.window.focused_viewport().buffer_id;
    let changed = try_move(&mut app.window, direction);

    if changed {
        let new_focused_id = app.window.focused_viewport().buffer_id;
        if old_focused_id != new_focused_id {
            // Hide cursor on old focused viewport
            if let Some(old_vp) = app::get_viewport_by_buffer_id_mut(
                &mut app.window,
                old_focused_id,
            ) {
                old_vp.hide_cursor = true;
            }
            // Show cursor on new focused viewport
            app.window.focused_viewport_mut().hide_cursor = false;
        }
    }

    Vec::new()
}
```

### Step 4: Add comprehensive tests

Test the following scenarios:

**Single-level Horizontal (existing tests, adapted):**
- `Down` on `Horizontal { focus: First }` → focus becomes `Second`.
- `Up` on `Horizontal { focus: Second }` → focus becomes `First`.
- `Left`/`Right` on `Horizontal` → no-op.

**Single-level Vertical:**
- `Right` on `Vertical { focus: First }` → focus becomes `Second`.
- `Left` on `Vertical { focus: Second }` → focus becomes `First`.
- `Up`/`Down` on `Vertical` → no-op.

**Nested: Vertical inside Horizontal:**
```
Horizontal {
    first: Vertical { first: Dir_A, second: Dir_B, focus: First },
    second: Tasks,
    focus: First,
}
```
- Focus on Dir_A, press `Right` → focus moves to Dir_B (within the Vertical).
- Focus on Dir_A, press `Down` → focus moves to Tasks (crosses from Horizontal first to second).
- Focus on Dir_B, press `Down` → focus moves to Tasks.
- Focus on Tasks, press `Up` → focus moves to Dir_A (enters Vertical from top, focuses first).

**Nested: Horizontal inside Vertical:**
```
Vertical {
    first: Horizontal { first: Dir_A, second: Tasks, focus: First },
    second: Dir_B,
    focus: First,
}
```
- Focus on Dir_A, press `Right` → focus moves to Dir_B (crosses from Vertical first to second).
- Focus on Dir_A, press `Down` → focus moves to Tasks (within the Horizontal).
- Focus on Dir_B, press `Left` → focus moves to Dir_A (enters Horizontal from right, focuses second/bottom... or first/top based on entry direction).

**Leaf root (no splits):**
- All directions are no-ops.

**Cursor visibility:**
- After any focus change, the old focused viewport has `hide_cursor = true` and the new focused viewport has `hide_cursor = false`.

### Step 5: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

### Nested Vertical inside Horizontal — focus movement
```
+--------+--------+
| Dir_A  | Dir_B  |   <- Vertical split (top half of Horizontal)
+--------+--------+
|     Tasks        |   <- Tasks (bottom half of Horizontal)
+------------------+

Focus on Dir_A:
  Ctrl+l (Right) → Dir_B  (Vertical: First → Second)
  Ctrl+j (Down)  → Tasks  (Horizontal: First → Second)
  Ctrl+h (Left)  → no-op  (already leftmost)
  Ctrl+k (Up)    → no-op  (already topmost)

Focus on Dir_B:
  Ctrl+h (Left)  → Dir_A  (Vertical: Second → First)
  Ctrl+j (Down)  → Tasks  (Horizontal: First → Second)
  Ctrl+l (Right) → no-op  (already rightmost)

Focus on Tasks:
  Ctrl+k (Up)    → Dir_A  (Horizontal: Second → First, enters Vertical from top → First)
  Ctrl+l (Right) → no-op  (no Vertical ancestor with room to move right)
```

### Deep nesting — three levels
```
Horizontal {
    first: Vertical {
        first: Dir_A,
        second: Horizontal {
            first: Dir_B,
            second: Dir_C,
            focus: First,
        },
        focus: First,
    },
    second: Tasks,
    focus: First,
}

Focus on Dir_A:
  Right → Dir_B  (Vertical first→second, enter Horizontal from direction Right → focus first/top)
  Down  → Tasks  (Horizontal first→second)

Focus on Dir_B:
  Left  → Dir_A  (Vertical second→first)
  Down  → Dir_C  (inner Horizontal first→second)
  Right → no-op  (no Vertical ancestor to the right)

Focus on Dir_C:
  Up    → Dir_B  (inner Horizontal second→first)
  Left  → Dir_A  (Vertical second→first)
  Down  → Tasks  (outer Horizontal first→second)
```

## Notes

- The `try_move` + `enter_from` pattern is a common approach for tree-based focus navigation (similar to how tmux and vim handle split pane focus). It naturally handles any depth of nesting.
- `enter_from` decides which child to focus when entering a subtree. The choice depends on the direction of movement — this ensures the user's spatial intuition is preserved (e.g., pressing `Down` into a `Vertical` split focuses the top-left leaf, not an arbitrary one).
- Cursor visibility is managed in two places: (1) `focus::change` toggles `hide_cursor` in the model for the old/new focused viewports, (2) `render_buffer_slot` applies a render-time visual override. Both are needed — the model update ensures correct state for non-rendering code paths, and the render-time override handles the multi-viewport case in `Directory` windows (parent and preview panes always have hidden cursors).
- The `get_viewport_by_buffer_id_mut` function (from `update/app.rs`) is used to find and update the old focused viewport after the tree walk changes focus. This is necessary because the old viewport may be in any branch of the tree.
- Existing tests in `focus.rs` will need minor updates to adapt to the new function structure, but their assertions should remain the same since the behavior for single-level splits is unchanged.
