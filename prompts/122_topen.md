# Overview

The `:topen` feature adds a task window to yeet — a split pane that lists running tasks, supports cancellation via `dd`, and live-updates as tasks start and end. The user types `:topen` to open a horizontal split with directory panes on top and a task list on the bottom, navigates between panes with `Ctrl+h/j/k/l`, and closes the task window with `:q` or `:tclose`.

The implementation is split into 9 sequential prompts, each leaving the program in a compilable and functional state:

1. [Prompt 1: Add `Window::Tasks` variant, `Buffer::Tasks` variant, and `SplitFocus` enum to the model](#prompt-1-add-windowtasks-variant-buffertasks-variant-and-splitfocus-enum-to-the-model) — `done`
2. [Prompt 2: Implement `Window::Horizontal` and `Window::Tasks` in all `todo!()` sites](#prompt-2-implement-windowhorizontal-and-windowtasks-in-all-todo-sites) — `done`
3. [Prompt 3: Add `FocusDirection` message, `Ctrl+h/j/k/l` keybindings, and focus switching logic](#prompt-3-add-focusdirection-message-ctrlhjkl-keybindings-and-focus-switching-logic) — `done`
4. [Prompt 4: Implement `:topen` command](#prompt-4-implement-topen-command) — `done`
5. [Prompt 5: Render the `Window::Tasks` and `Buffer::Tasks` types](#prompt-5-render-the-windowtasks-and-buffertasks-types) — `done`
6. [Prompt 6: Handle `dd` in the task window to cancel tasks](#prompt-6-handle-dd-in-the-task-window-to-cancel-tasks) — `done`
7. [Prompt 7: Live-update the task buffer on `TaskStarted` / `TaskEnded`](#prompt-7-live-update-the-task-buffer-on-taskstarted--taskended) — `planned`
8. [Prompt 8: Implement `:q` to close focused window, `:qa` / `:qa!` to quit](#prompt-8-implement-q-to-close-focused-window-qa--qa-to-quit) — `planned`
9. [Prompt 9: Edge cases, polish, and safety](#prompt-9-edge-cases-polish-and-safety) — `planned`

---

# Prompt 1: Add `Window::Tasks` variant, `Buffer::Tasks` variant, and `SplitFocus` enum to the model

**Goal**: Introduce the new data types for the task window feature without any behavior changes.

**State**: `done`

**Motivation**: The `:topen` command needs a way to represent a task window in the window tree and a task-specific buffer type. Adding the types first — with all match arms compiling — isolates model changes from logic changes, keeping each step reviewable and the program functional.

## Requirements

- Add a `SplitFocus` enum with `First` (default) and `Second` variants.
- Change `Window::Horizontal` from a tuple variant to a struct variant with `first`, `second`, and `focus` fields.
- Add `Window::Tasks(ViewPort)` as a new leaf variant.
- Add a `TasksBuffer` struct wrapping a `TextBuffer`.
- Add `Buffer::Tasks(TasksBuffer)` to the `Buffer` enum.
- Update every `Window::Horizontal(_, _)` pattern match to the new struct syntax (`Window::Horizontal { .. }`), keeping existing `todo!()` bodies.
- Add `Window::Tasks(_)` arms with `todo!()` to every exhaustive match on `Window`.
- Handle `Buffer::Tasks` in every exhaustive match on `Buffer` (same as `Buffer::Empty` — return early / no-op).
- All existing tests continue to pass. New tests verify type construction and pattern matching.

## Exclusions

- Do NOT implement any `Horizontal` or `Tasks` logic (leave `todo!()`).
- Do NOT add commands, keybindings, or rendering.
- Do NOT change runtime behavior — this is a types-only change.

## Context

- @yeet-frontend/src/model/mod.rs — `Window` enum (line ~78), `Buffer` enum (line ~160), `ViewPort`, `TextBuffer`, `Contents`.
- @yeet-frontend/src/update/app.rs — `get_focused_current_mut`, `get_focused_directory_viewports`, `get_focused_directory_viewports_mut`, `get_viewport_by_buffer_id_mut`.
- @yeet-frontend/src/update/window.rs — `set_buffer_vp`.
- @yeet-frontend/src/update/buffers.rs — `update`.
- @yeet-frontend/src/view/mod.rs — `model`.
- @yeet-frontend/src/view/buffer.rs — `view`, `render_buffer_slot`.
- @yeet-frontend/src/view/statusline.rs — `view()`.
- @yeet-frontend/src/update/save.rs — exhaustive `Buffer` match.
- @yeet-frontend/src/update/mode.rs — `match app::get_focused_current_mut(app)` destructuring `Buffer`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Add `SplitFocus` enum** in `yeet-frontend/src/model/mod.rs`:

   ```rust
   #[derive(Clone, Debug, Default, PartialEq, Eq)]
   pub enum SplitFocus {
       #[default]
       First,
       Second,
   }
   ```

2. **Change `Window::Horizontal`** from a tuple variant to a struct variant and add `Window::Tasks`:

   ```rust
   pub enum Window {
       Horizontal {
           first: Box<Window>,
           second: Box<Window>,
           focus: SplitFocus,
       },
       Directory(ViewPort, ViewPort, ViewPort),
        Tasks(ViewPort),
    }
    ```

   Remove the `#[allow(dead_code)]` attribute since `Horizontal` will no longer be dead code once the feature is complete.

3. **Add `TasksBuffer`** struct:

   ```rust
   pub struct TasksBuffer {
       pub buffer: TextBuffer,
   }
   ```

4. **Add `Buffer::Tasks(TasksBuffer)`** variant to the existing `Buffer` enum.

5. **Update `Window::Horizontal` pattern matches** — all 8 `todo!()` sites change syntax only:
   - `yeet-frontend/src/model/mod.rs` — `get_height()`
   - `yeet-frontend/src/update/app.rs` — `get_focused_current_mut`, `get_focused_directory_viewports`, `get_focused_directory_viewports_mut`, `get_viewport_by_buffer_id_mut`
   - `yeet-frontend/src/update/window.rs` — `set_buffer_vp`
   - `yeet-frontend/src/update/buffers.rs` — `update`
   - `yeet-frontend/src/view/mod.rs` — `model`
   - `yeet-frontend/src/view/buffer.rs` — `view`

6. **Add `Window::Tasks(_)` arms** with `todo!()` to every exhaustive match listed in step 5.

7. **Handle `Buffer::Tasks`** in every exhaustive match on `Buffer` (treat as `Buffer::Empty` — early return / no-op):
   - `yeet-frontend/src/view/buffer.rs` — `render_buffer_slot`
   - `yeet-frontend/src/view/statusline.rs` — `view()`
   - `yeet-frontend/src/update/save.rs`
   - `yeet-frontend/src/update/mode.rs` — every `match` that destructures `Buffer`
   - Any other exhaustive match — grep for `Buffer::Directory`, `Buffer::Empty`, etc.

8. **Add tests**:
   - `SplitFocus::default() == SplitFocus::First`
   - Construct `Window::Tasks(ViewPort::default())` and pattern-match it back.
   - Construct `Window::Horizontal { first: Box::new(Window::Directory(...)), second: Box::new(Window::Tasks(...)), focus: SplitFocus::First }` — verify the recursive tree compiles.
   - Construct `TasksBuffer` with a default `TextBuffer`, wrap in `Buffer::Tasks(...)`, pattern-match.

9. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// SplitFocus default
assert_eq!(SplitFocus::default(), SplitFocus::First);

// Window::Tasks construction
let task_window = Window::Tasks(ViewPort::default());
assert!(matches!(task_window, Window::Tasks(_)));

// Horizontal tree
let tree = Window::Horizontal {
    first: Box::new(Window::Directory(
        ViewPort::default(), ViewPort::default(), ViewPort::default(),
    )),
    second: Box::new(Window::Tasks(ViewPort::default())),
    focus: SplitFocus::First,
};
assert!(matches!(tree, Window::Horizontal { .. }));

// Buffer::Tasks
let buf = Buffer::Tasks(TasksBuffer { buffer: TextBuffer::default() });
assert!(matches!(buf, Buffer::Tasks(_)));
```

## Notes

- The `#[allow(dead_code)]` on the `Window` enum exists because `Horizontal` was never constructed. Removing it now is safe since the variant will be constructed in Prompt 4.
- Pattern match updates are purely mechanical — change `Horizontal(a, b)` to `Horizontal { .. }` and keep the `todo!()` body.

---

# Prompt 2: Implement `Window::Horizontal` and `Window::Tasks` in all `todo!()` sites

**Goal**: Replace every `todo!()` on `Window::Horizontal` and `Window::Tasks` with real implementations, making the window tree fully functional for recursive splits and task windows.

**State**: `done`

**Motivation**: The window tree must support splits and task leaves before any user-facing feature (`:topen`, focus switching) can work. Implementing all `todo!()` sites in one prompt ensures the infrastructure is complete and testable end-to-end.

## Requirements

- Replace all 9 `todo!()` sites with working logic for both `Horizontal` and `Tasks` arms.
- `Horizontal` layout splits the area vertically (top/bottom, 50/50).
- `Tasks` viewport behaves like a single-pane leaf (height, width, x, y from area).
- Focus-aware functions recurse into the focused child of `Horizontal`.
- Buffer-id collection recursively walks the entire tree.
- `get_focused_directory_viewports`, `_mut`, and `_buffer_ids` return `Option` — `None` when the focused leaf is a Tasks window. All callers are updated accordingly.
- All existing tests continue to pass.

## Exclusions

- Do NOT add commands, keybindings, or the `:topen` command.
- Do NOT add rendering for `Buffer::Tasks` content (that is Prompt 5).
- This prompt only makes the window tree infrastructure work.

## Context

- @yeet-frontend/src/model/mod.rs — `Window` enum, `SplitFocus`, `get_height()`.
- @yeet-frontend/src/update/app.rs — `get_focused_current_mut`, `get_focused_directory_viewports`, `get_focused_directory_viewports_mut`, `get_viewport_by_buffer_id_mut`, `get_buffer_path`.
- @yeet-frontend/src/update/window.rs — `set_buffer_vp`, uses `ratatui::layout::{Layout, Direction, Constraint}`.
- @yeet-frontend/src/update/buffers.rs — `update`, uses `HashSet<usize>` for referenced buffer ids.
- @yeet-frontend/src/view/mod.rs — `model` function (status line context).
- @yeet-frontend/src/view/buffer.rs — `view` function (recursive rendering).
- @yeet-frontend/src/update/navigate.rs — caller of `get_focused_directory_viewports_mut` and `_buffer_ids`.
- @yeet-frontend/src/update/path.rs — caller of `get_focused_directory_buffer_ids`.
- @yeet-frontend/src/update/cursor.rs — caller of `get_focused_directory_buffer_ids`.
- @yeet-frontend/src/update/selection.rs — caller of `get_focused_directory_viewports_mut`.
- @yeet-frontend/src/update/preview.rs — caller of `get_focused_directory_viewports_mut`.
- @yeet-frontend/src/update/enumeration.rs — caller of `get_focused_directory_buffer_ids`.
- @yeet-frontend/src/update/command/mod.rs — caller of `get_focused_directory_buffer_ids`.
- @yeet-frontend/src/action.rs — caller of `get_focused_directory_viewports` and `_buffer_ids`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Add `Window` methods in `yeet-frontend/src/model/mod.rs`

Add three methods to the `Window` impl block:

1a. **`focused_viewport(&self) -> &ViewPort`** — recursively follows `SplitFocus` to the focused leaf. For `Directory`, returns the middle viewport (current). For `Tasks`, returns the single viewport. For `Horizontal`, recurses into the focused child.

```rust
pub fn focused_viewport(&self) -> &ViewPort {
    match self {
        Window::Horizontal { first, second, focus } => match focus {
            SplitFocus::First => first.focused_viewport(),
            SplitFocus::Second => second.focused_viewport(),
        },
        Window::Directory(_, vp, _) => vp,
        Window::Tasks(vp) => vp,
    }
}
```

1b. **`focused_viewport_mut(&mut self) -> &mut ViewPort`** — mutable version of the above, same recursive structure.

1c. **`buffer_ids(&self) -> HashSet<usize>`** — recursively collects all buffer ids from the window tree. For `Horizontal`, unions both children. For `Directory`, returns the 3 viewport buffer ids. For `Tasks`, returns the single buffer id.

```rust
pub fn buffer_ids(&self) -> HashSet<usize> {
    match self {
        Window::Horizontal { first, second, .. } => {
            let mut ids = first.buffer_ids();
            ids.extend(second.buffer_ids());
            ids
        }
        Window::Directory(parent, current, preview) => {
            HashSet::from([parent.buffer_id, current.buffer_id, preview.buffer_id])
        }
        Window::Tasks(vp) => HashSet::from([vp.buffer_id]),
    }
}
```

1d. **`get_height`** — replace the `todo!()` arms:
- `Horizontal { first, second, .. }` → `first.get_height()? + second.get_height()?`
- `Tasks(vp)` → `Ok(vp.height)`

### Step 2: Restructure `get_focused_current_mut` in `yeet-frontend/src/update/app.rs`

Change the signature to accept `&mut Window` and `&mut Contents` directly instead of `&mut App`. This avoids destructuring `app` inside the method and makes the borrow split explicit at the call site:

```rust
pub fn get_focused_current_mut<'a>(
    window: &'a mut Window,
    contents: &'a mut Contents,
) -> (&'a mut ViewPort, &'a mut Buffer) {
    let vp = window.focused_viewport_mut();
    let focused_id = vp.buffer_id;
    match contents.buffers.get_mut(&focused_id) {
        Some(it) => (vp, it),
        None => panic!("focused viewport references non-existent buffer {}", focused_id),
    }
}
```

All 19 callers must be updated from `app::get_focused_current_mut(app)` to `app::get_focused_current_mut(&mut app.window, &mut app.contents)`. The split into `&mut app.window` and `&mut app.contents` at the call site satisfies the borrow checker because Rust can see these are disjoint fields of `App`.

### Step 3: Change `get_focused_directory_viewports` to return `Option`

Change return type from `(&ViewPort, &ViewPort, &ViewPort)` to `Option<(&ViewPort, &ViewPort, &ViewPort)>`. For `Horizontal`, recurse into the focused child. For `Tasks`, return `None`. For `Directory`, return `Some(...)`.

```rust
pub fn get_focused_directory_viewports(window: &Window) -> Option<(&ViewPort, &ViewPort, &ViewPort)> {
    match window {
        Window::Horizontal { first, second, focus } => match focus {
            SplitFocus::First => get_focused_directory_viewports(first),
            SplitFocus::Second => get_focused_directory_viewports(second),
        },
        Window::Directory(parent, current, preview) => Some((parent, current, preview)),
        Window::Tasks(_) => None,
    }
}
```

Passing `&Window` directly enables natural recursion without needing a separate helper. All callers change from `get_focused_directory_viewports(app)` to `get_focused_directory_viewports(&app.window)`.

### Step 4: Change `get_focused_directory_viewports_mut` to return `Option`

Same approach as Step 3, mutable version. The existing signature already takes `window: &mut Window` — just change the return type to `Option`.

### Step 5: Change `get_focused_directory_buffer_ids` to return `Option`

```rust
pub fn get_focused_directory_buffer_ids(window: &Window) -> Option<(usize, usize, usize)> {
    let (parent, current, preview) = get_focused_directory_viewports(window)?;
    Some((parent.buffer_id, current.buffer_id, preview.buffer_id))
}
```

All callers change from `get_focused_directory_buffer_ids(app)` to `get_focused_directory_buffer_ids(&app.window)`.

### Step 6: Update all callers of `get_focused_directory_viewports`, `_mut`, and `_buffer_ids`

Each caller must handle the new `Option` return. Strategy per caller:

- **Callers that fundamentally require directory viewports** (navigation, path reset, preview set): use `.expect("requires directory window")` or early-return if appropriate.
- **Callers that can gracefully handle None** (commands returning `Vec<Action>`, actions): propagate `None` and return `Vec::new()` or skip the operation.

Known callers (~14 sites):
- `action.rs` — `get_focused_directory_viewports`, `get_focused_directory_buffer_ids`: return early / skip if `None`
- `update/navigate.rs` — `get_focused_directory_viewports_mut`, `get_focused_directory_buffer_ids`: `.expect(...)` (navigation is fundamentally directory-specific)
- `update/path.rs` — `get_focused_directory_buffer_ids`: early-return if `None`
- `update/cursor.rs` — `get_focused_directory_buffer_ids`: early-return if `None`
- `update/selection.rs` — `get_focused_directory_viewports_mut`: `.expect(...)` (preview refresh is directory-specific)
- `update/preview.rs` — `get_focused_directory_viewports_mut`: `.expect(...)` (preview is directory-specific)
- `update/command/mod.rs` — `get_focused_directory_buffer_ids`: propagate `None` (already returns `Option<&Path>`)
- `update/enumeration.rs` — `get_focused_directory_buffer_ids`: early-return if `None`
- `update/junkyard.rs` — `get_focused_directory_buffer_ids`: early-return if `None`
- `update/open.rs` — (if it calls `get_focused_directory_buffer_ids`): early-return if `None`
- Tests in `update/buffers.rs`, `update/mode.rs`, `update/path.rs`: `.unwrap()` / `.expect(...)` (tests always use directory windows)

### Step 7: Implement `get_viewport_by_buffer_id_mut`

Recursive search through the window tree:

```rust
pub fn get_viewport_by_buffer_id_mut(window: &mut Window, buffer_id: usize) -> Option<&mut ViewPort> {
    match window {
        Window::Horizontal { first, second, .. } => {
            get_viewport_by_buffer_id_mut(first, buffer_id)
                .or_else(|| get_viewport_by_buffer_id_mut(second, buffer_id))
        }
        Window::Directory(parent, current, preview) => {
            if parent.buffer_id == buffer_id { Some(parent) }
            else if current.buffer_id == buffer_id { Some(current) }
            else if preview.buffer_id == buffer_id { Some(preview) }
            else { None }
        }
        Window::Tasks(vp) => {
            if vp.buffer_id == buffer_id { Some(vp) } else { None }
        }
    }
}
```

### Step 8: Restructure `set_buffer_vp` in `yeet-frontend/src/update/window.rs`

Match on the window variant at the top. Move the 3-column horizontal layout into the `Directory` arm. `Horizontal` splits the area vertically 50/50 and recurses. `Tasks` sets viewport dimensions directly.

```rust
fn set_buffer_vp(window: &mut Window, area: Rect) -> Result<(), AppError> {
    match window {
        Window::Horizontal { first, second, .. } => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(Constraint::from_ratios([(1, 2), (1, 2)]))
                .split(area);
            set_buffer_vp(first, layout[0])?;
            set_buffer_vp(second, layout[1])?;
            Ok(())
        }
        Window::Directory(parent, current, preview) => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
                .split(area);
            // ... assign layout[0/1/2] dimensions to parent/current/preview viewports
            Ok(())
        }
        Window::Tasks(vp) => {
            vp.height = area.height;
            vp.width = area.width;
            vp.x = area.x;
            vp.y = area.y;
            Ok(())
        }
    }
}
```

### Step 9: Replace `view::model` match with `Window::focused_viewport()`

In `yeet-frontend/src/view/mod.rs`, replace the inline match:

```rust
// Before:
let (focused_id, focused_vp) = match &model.app.window {
    Window::Horizontal { .. } => todo!(),
    Window::Directory(_, vp, _) => (&vp.buffer_id, vp),
    Window::Tasks(_) => todo!(),
};

// After:
let focused_vp = model.app.window.focused_viewport();
let focused_id = &focused_vp.buffer_id;
```

### Step 10: Extract `render_window` helper in `yeet-frontend/src/view/buffer.rs`

Replace the current `view()` function with a recursive `render_window` helper that takes `(&Window, &HashMap<usize, Buffer>)` instead of `&App`:

```rust
pub fn view(mode: &Mode, app: &App, frame: &mut Frame, h_off: u16, v_off: u16) {
    render_window(mode, &app.window, &app.contents.buffers, frame, h_off, v_off);
}

fn render_window(
    mode: &Mode,
    window: &Window,
    buffers: &HashMap<usize, Buffer>,
    frame: &mut Frame,
    h_off: u16,
    v_off: u16,
) {
    match window {
        Window::Horizontal { first, second, .. } => {
            render_window(mode, first, buffers, frame, h_off, v_off);
            render_window(mode, second, buffers, frame, h_off, v_off);
        }
        Window::Directory(parent, current, preview) => {
            render_buffer_slot(mode, frame, parent, buffers.get(&parent.buffer_id), h_off, v_off);
            render_buffer_slot(mode, frame, current, buffers.get(&current.buffer_id), h_off, v_off);
            render_buffer_slot(mode, frame, preview, buffers.get(&preview.buffer_id), h_off, v_off);
        }
        Window::Tasks(vp) => {
            render_buffer_slot(mode, frame, vp, buffers.get(&vp.buffer_id), h_off, v_off);
        }
    }
}
```

### Step 11: Simplify `buffers::update` with `Window::buffer_ids()`

Replace the match + early-return with the new method:

```rust
pub fn update(app: &mut App) {
    let referenced = app.window.buffer_ids();
    // ... existing stale image cleanup logic unchanged
}
```

### Step 12: Add tests

- `Window::get_height` for `Horizontal { Tasks(h=10), Directory(h=15) }` → returns `Ok(25)`.
- `Window::focused_viewport` on `Horizontal { Directory, Tasks, focus: Second }` → returns the Tasks viewport.
- `Window::focused_viewport` on `Horizontal { Directory, Tasks, focus: First }` → returns the Directory middle viewport.
- `Window::buffer_ids` on a nested tree returns all buffer ids.
- `set_buffer_vp` with a `Horizontal` tree: both children get correct y-offsets (first starts at `area.y`, second at `area.y + area.height/2`).
- `get_viewport_by_buffer_id_mut` recursively finds a viewport in the second child.
- `get_focused_current_mut` on `Horizontal { Directory, Tasks, focus: Second }` returns the tasks viewport and buffer.
- `get_focused_directory_viewports` returns `None` when focused on a `Tasks` leaf.
- `get_focused_directory_viewports` returns `Some(...)` when focused on a `Directory` through `Horizontal`.

### Step 13: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

```rust
// get_height with horizontal split
let tree = Window::Horizontal {
    first: Box::new(Window::Tasks(ViewPort { height: 10, ..Default::default() })),
    second: Box::new(Window::Directory(
        ViewPort::default(),
        ViewPort { height: 15, ..Default::default() },
        ViewPort::default(),
    )),
    focus: SplitFocus::First,
};
assert_eq!(tree.get_height().unwrap(), 25);

// focused_viewport follows SplitFocus
let vp = tree.focused_viewport();
assert_eq!(vp.height, 10); // Tasks viewport (first child, focused)

// buffer_ids collects from all leaves
let ids = tree.buffer_ids();
assert_eq!(ids.len(), 4); // 1 from Tasks + 3 from Directory

// get_focused_directory_viewports returns None for Tasks leaf
// (assuming focus is on the Tasks child)
assert!(get_focused_directory_viewports(&app_with_tasks_focused.window).is_none());

// set_buffer_vp layout
// Given area { x: 0, y: 0, width: 80, height: 40 }:
// first child gets  { x: 0, y: 0,  width: 80, height: 20 }
// second child gets { x: 0, y: 20, width: 80, height: 20 }
```

## Notes

- `Window::focused_viewport()` and `focused_viewport_mut()` unify the needs of `get_focused_current_mut` (Step 2) and `view::model` (Step 9). Implementing them as methods on `Window` keeps the logic close to the data and avoids duplication.
- `Window::buffer_ids()` fits alongside the existing `get_height()` method as a pure query on the window tree.
- The `get_focused_current_mut` signature changes from `(app: &mut App)` to `(window: &mut Window, contents: &mut Contents)`. This makes the borrow split explicit at each call site (`&mut app.window, &mut app.contents`), avoiding hidden destructuring inside the method. All 19 callers must be updated.
- `get_focused_directory_viewports` and `get_focused_directory_buffer_ids` change from taking `&App` to `&Window`, matching the Step 2 pattern of passing the narrowest type needed. Combined with the `Option` return, this is a significant API change affecting ~14 callers. Callers that fundamentally require directory viewports (navigation, preview) use `.expect(...)`. Callers that can handle a Tasks focus (commands, cursor, actions) propagate `None` gracefully.
- The 50/50 split ratio in `set_buffer_vp` may be adjusted in Prompt 5 after visual verification.
- The `render_window` helper in `view/buffer.rs` takes `&HashMap<usize, Buffer>` instead of `&App` to enable recursion — each recursive call passes the same buffers reference but a different `&Window` subtree.

---

# Prompt 3: Add `FocusDirection` message, `Ctrl+h/j/k/l` keybindings, and focus switching logic

**Goal**: Add keybindings and logic to move focus between windows in a split layout.

**State**: `done`

**Motivation**: Once splits exist (Prompt 4), users need a way to move focus between the directory panes and the task window. Wiring this up now — while no splits exist yet — keeps the change isolated and testable.

## Requirements

- Add `FocusDirection` enum (`Up`, `Down`, `Left`, `Right`) to `yeet-keymap`.
- Add `FocusDirection(FocusDirection)` variant to `KeymapMessage`.
- Bind `Ctrl+h/j/k/l` in Navigation and Normal modes to the corresponding directions.
- Implement `focus::change` in `yeet-frontend` that updates `SplitFocus` and toggles cursor visibility.
- For a single-level `Horizontal` (top/bottom): `Down` → `Second`, `Up` → `First`, `Left`/`Right` → no-op.
- For non-split windows (`Directory` or `Tasks` at root): all directions are no-ops.

## Exclusions

- Do NOT handle nested/multi-level splits beyond one level of `Horizontal`.
- Do NOT add `:topen` or any other commands.

## Context

- @yeet-keymap/src/message.rs — `KeymapMessage` enum, `QuitMode`, derive conventions.
- @yeet-keymap/src/map.rs — keybinding definitions, shared Navigation + Normal section.
- @yeet-frontend/src/update/mod.rs — message dispatch, `mod` declarations.
- @yeet-frontend/src/model/mod.rs — `App`, `Window`, `SplitFocus`, `ViewPort`.
- @yeet-keymap/tests/lib_tests.rs — existing keymap tests (test naming conventions, mode setup).
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Add `FocusDirection`** in `yeet-keymap/src/message.rs`:

   ```rust
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub enum FocusDirection {
       Up,
       Down,
       Left,
       Right,
   }
   ```

   Add `FocusDirection(FocusDirection)` variant to `KeymapMessage`.

2. **Add keybindings** in `yeet-keymap/src/map.rs` (shared Navigation + Normal section):
   - `Ctrl+h` → `KeymapMessage::FocusDirection(FocusDirection::Left)`
   - `Ctrl+j` → `KeymapMessage::FocusDirection(FocusDirection::Down)`
   - `Ctrl+k` → `KeymapMessage::FocusDirection(FocusDirection::Up)`
   - `Ctrl+l` → `KeymapMessage::FocusDirection(FocusDirection::Right)`
   - Check for existing `Ctrl+h/j/k/l` bindings. Resolve conflicts if any.

3. **Create `yeet-frontend/src/update/focus.rs`**:

   ```rust
   pub fn change(app: &mut App, direction: &FocusDirection) -> Vec<Action>
   ```

   - `Window::Horizontal` at root: `Down` → focus `Second`, `Up` → focus `First`. `Left`/`Right` → no-op.
   - When changing focus: hide cursor on old focused leaf, show cursor on new one.
   - `Window::Directory` or `Window::Tasks` at root: all directions are no-ops.
   - Only handle one level of `Horizontal` (`:topen` creates at most one split).

4. **Wire into dispatch** in `yeet-frontend/src/update/mod.rs`:
   - Add `mod focus;`.
   - Add handler: `KeymapMessage::FocusDirection(direction) => focus::change(app, direction)`.

5. **Add tests**:
   - In `yeet-keymap/tests/lib_tests.rs`: `Ctrl+j` in Navigation mode → `FocusDirection(Down)`.
   - In `yeet-keymap/tests/lib_tests.rs`: `Ctrl+k` in Normal mode → `FocusDirection(Up)`.
   - In `yeet-frontend/src/update/focus.rs`:
     - `change` on `Horizontal { Directory, Tasks, focus: First }` with `Down` → focus becomes `Second`.
     - `change` on same with `Up` → no change (already on First).
     - `change` on a plain `Directory` root → no change for any direction.
     - Cursor visibility toggles correctly on focus change.

6. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// Focus change: Down on Horizontal moves to Second
let mut app = App::with_window(Window::Horizontal {
    first: Box::new(directory_window()),
    second: Box::new(task_window()),
    focus: SplitFocus::First,
});
focus::change(&mut app, &FocusDirection::Down);
// app.window is now Horizontal { ..., focus: SplitFocus::Second }
```

## Notes

- Since no splits can be created yet, these keybindings are inert in practice. They become active once Prompt 4 adds `:topen`.
- Check for existing `Ctrl+h/j/k/l` conflicts carefully — these are common bindings.

---

# Prompt 4: Implement `:topen` command

**Goal**: Add the `:topen` command that creates a task buffer, wraps the current window in a `Window::Horizontal` split with a `Window::Tasks` as the second child, and focuses the task window.

**State**: `done`

**Motivation**: This is the core user-facing entry point for the task window feature. The user types `:topen` and sees a split appear with a list of running tasks.

## Requirements

- `:topen` creates a `Buffer::Tasks` with lines built from `tasks.running` (sorted by id, formatted as `"{id:<4} {external_id}"`).
- The current window is wrapped in `Window::Horizontal { first: old_window, second: Window::Tasks(...), focus: SplitFocus::Second }`.
- If a `Window::Tasks` already exists in the tree, `:topen` switches focus to it instead of creating a duplicate.
- The task buffer is properly registered in `app.contents.buffers`.

## Exclusions

- Do NOT implement cursor visibility (Prompt 5).
- Do NOT implement rendering (Prompt 5).
- Do NOT implement `dd` cancellation (Prompt 6).
- Do NOT implement live updates on `TaskStarted`/`TaskEnded` (Prompt 7).

## Context

- @yeet-frontend/src/update/command/task.rs — existing `delete` function (task cancellation by ID).
- @yeet-frontend/src/update/command/mod.rs — command dispatch (`execute` function, match arms).
- @yeet-frontend/src/model/mod.rs — `App`, `Window`, `Buffer`, `TasksBuffer`, `Tasks`, `CurrentTask`, `Contents`, `ViewPort`, `SplitFocus`.
- @yeet-buffer — `TextBuffer`, `BufferLine`, `Ansi`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Add `task::open`** in `yeet-frontend/src/update/command/task.rs`:

   ```rust
   pub fn open(app: &mut App, tasks: &Tasks) -> Vec<Action>
   ```

   - Build task lines from `tasks.running`: sort by `task.id`, format each as `"{id:<4} {external_id}"` using `BufferLine { content: Ansi::new(&formatted), ..Default::default() }`.
   - Allocate a new buffer_id from `app.contents`, insert `Buffer::Tasks(TasksBuffer { buffer: TextBuffer { lines, ..Default::default() } })`.
   - Create a `ViewPort`: `ViewPort { buffer_id, hide_cursor: false, show_border: true, ..Default::default() }`.
   - Take the current `app.window` (via `std::mem::take` or `std::mem::replace`), wrap it:

     ```rust
     app.window = Window::Horizontal {
         first: Box::new(old_window),
         second: Box::new(Window::Tasks(task_viewport)),
         focus: SplitFocus::Second,
     };
     ```

   - Hide cursor on the old window's focused leaf viewport, show cursor on the task viewport.
   - If the window tree already contains a `Window::Tasks` (check recursively), just switch focus to it instead of creating a new one.

2. **Add command dispatch** in `yeet-frontend/src/update/command/mod.rs`:
   - Add match arm: `("topen", "") => task::open(app, &state.tasks)` wrapped in `add_change_mode`.

3. **Add tests**:
   - `open` creates a `Horizontal { Directory, Tasks }` tree with correct focus.
   - `open` with tasks: task buffer lines match `"{id:<4} {external_id}"` formatting.
   - `open` with no running tasks: task window created with empty buffer.
   - `open` when task window already exists: focus switches without creating duplicates.
   - Task buffer is in `app.contents.buffers` with the correct buffer_id.

4. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// Task line formatting
// Given tasks: [CurrentTask { id: 1, external_id: "rg foo" }, CurrentTask { id: 12, external_id: "fd bar" }]
// Sorted by id, formatted lines:
// "1    rg foo"
// "12   fd bar"

// Window tree after :topen
// Window::Horizontal {
//     first: Box::new(Window::Directory(parent_vp, current_vp, preview_vp)),
//     second: Box::new(Window::Tasks(task_vp)),
//     focus: SplitFocus::Second,
// }
```

## Notes

- `Window` needs a `Default` impl for `std::mem::take` to work. If not already added, add `impl Default for Window` returning `Window::Directory(Default::default(), Default::default(), Default::default())`. (This may be deferred to Prompt 8 if `:q` needs it first — check if needed here.)
- The idempotency check (don't create nested splits) is important — users may reflexively type `:topen` again.

---

# Prompt 5: Render the `Window::Tasks` and `Buffer::Tasks` types

**Goal**: Make the task window visible when `:topen` is run — directory panes on top, task list on the bottom.

**State**: `done`

**Motivation**: After Prompt 4, `:topen` creates the split in the model but nothing renders. This prompt connects the model to the view so the user actually sees the task window.

## Requirements

- `render_buffer_slot` handles `Buffer::Tasks` by rendering it the same way as `Buffer::Content` (via `yeet_buffer::view()`).
- The status line shows a "Tasks" label (or "Tasks: N running") when the task window is focused.
- The `Horizontal` split renders both children with correct layout (directory panes on top, task list on bottom).
- Cursor visibility for focused vs unfocused windows is handled purely at render time as a visual override — **no viewport state is mutated**. The rendering code determines whether a viewport belongs to the focused leaf window and passes a cloned viewport with `hide_cursor` / `hide_cursor_line` set accordingly to the buffer view function. Unfocused windows render with cursor hidden; the focused window renders with cursor shown. This is a view-only concern and must not alter any `ViewPort` in the model.

## Exclusions

- Do NOT add task cancellation (`dd`) — that is Prompt 6.
- Do NOT add live updates — that is Prompt 7.
- Do NOT alter viewport state (`hide_cursor`, `hide_cursor_line`) outside of rendering — cursor visibility is a render-time visual only.

## Context

- @yeet-frontend/src/view/buffer.rs — `render_buffer_slot`, `view` function, `buffer_view` helper.
- @yeet-frontend/src/view/statusline.rs — `view()`, match on `Buffer` variants.
- @yeet-frontend/src/update/window.rs — `set_buffer_vp` (layout constraints from Prompt 2).
- @yeet-frontend/src/view/mod.rs — `model` function (focus context for status line).
- @yeet-buffer — `yeet_buffer::view()` for rendering `TextBuffer`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Render task buffer** in `yeet-frontend/src/view/buffer.rs`:
   - In `render_buffer_slot`, add handling for `Buffer::Tasks(tasks_buf)`: render using `yeet_buffer::view()` on `tasks_buf.buffer`, same pattern as `Buffer::Content`.

2. **Cursor visibility as a render-time visual** in `yeet-frontend/src/view/buffer.rs`:
   - Pass the focused `buffer_id` (from `Window::focused_viewport().buffer_id`) down through `render_window` and into `render_buffer_slot`.
   - In `render_buffer_slot`, compare the viewport's `buffer_id` against the focused `buffer_id`. If they differ, clone the viewport and set `hide_cursor = true` / `hide_cursor_line = true` on the clone before passing it to the buffer view function. If they match, render with the viewport as-is.
   - This is purely visual — no `ViewPort` in the model is mutated. The clone is a temporary local used only for the render call.

3. **Status line** in `yeet-frontend/src/view/statusline.rs`:
   - Add `Buffer::Tasks(_)` to the match in `view()`. Render a simple "Tasks" label or "Tasks: N running".

4. **Verify recursive rendering**: `view::buffer::view` (from Prompt 2) should correctly render both children of the `Horizontal` split. Directory panes appear in the top half, task list in the bottom half.

5. **Verify/adjust layout**: `set_buffer_vp` (from Prompt 2) splits 50/50. Visually verify this. Consider whether the task window should be smaller (e.g., 70/30). Adjust constraints in `set_buffer_vp` if needed.

6. **Add tests**:
   - After `open` + `window::update(app, area)`, all viewports (directory parent/current/preview + task) have non-zero dimensions.
   - Task viewport's `y` offset is greater than directory viewports' `y` offset (it is below).
   - `Window::get_height()` on the `Horizontal` tree returns the full area height.

7. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```
+-----------------------------------------------+
|  parent  |   current    |      preview         |  <- Directory panes (top)
|          |              |                      |
+-----------------------------------------------+
|  1    rg foo                                   |  <- Task list (bottom)
|  12   fd bar                                   |
+-----------------------------------------------+
```

## Notes

- At this point `:topen` should produce a visible split with the task list rendered.
- Cursor visibility is strictly a rendering concern. The `ViewPort` fields `hide_cursor` and `hide_cursor_line` in the model are **not** toggled when focus changes — instead, `render_buffer_slot` determines at render time whether the viewport is focused and applies a visual-only override via a cloned viewport. This keeps the model clean and avoids state synchronization bugs when focus changes.

---

# Prompt 6: Handle `dd` in the task window to cancel tasks

**Goal**: When the task window is focused and `dd` is pressed, cancel the task at the cursor and mark it visually with dim/strikethrough text.

**State**: `done`

**Motivation**: Users need a way to cancel running tasks directly from the task window. The `dd` keybinding is natural (delete the line = cancel the task). The cancelled line stays visible until `TaskEnded` removes it, providing visual feedback.

## Requirements

- `dd` on a task buffer line cancels the corresponding task via `token.cancel()`.
- Cancelled tasks are detected via `token.is_cancelled()` and displayed with ANSI strikethrough + dim styling.
- All other text modifications on the task buffer are blocked (no-op).
- No model changes needed — cancellation state is derived from `CancellationToken::is_cancelled()`.
- Mode change from Navigation → Normal (triggered by `dd`'s `force`) handles `Buffer::Tasks` gracefully (early return).

## Exclusions

- Do NOT implement live updates on `TaskStarted`/`TaskEnded` — that is Prompt 7.
- Do NOT implement `:tclose` — that is Prompt 9.

## Context

- @yeet-frontend/src/model/mod.rs — `CurrentTask` struct, `Tasks`.
- @yeet-frontend/src/update/modify.rs — `buffer()` function, `TextModification::DeleteLine`.
- @yeet-frontend/src/update/mode.rs — `change()`, `Buffer::Directory` arms.
- @yeet-buffer — `TextBuffer`, `BufferLine`, `Ansi`, cursor `vertical_index`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Handle `dd` in task buffer** in `yeet-frontend/src/update/modify.rs`:
   - Change `buffer()` signature to take `&mut State` (drop separate `mode` parameter, read `state.modes.current` internally).
   - In `buffer()`, match on focused buffer type:
     - `Buffer::Tasks(_)` + `TextModification::DeleteLine`: extract cursor `vertical_index`, sort tasks by id, find the task at that index, call `token.cancel()`. Call `refresh_tasks_buffer` to rebuild buffer lines. Return `Vec::new()`.
     - `Buffer::Tasks(_)` + any other modification: return `Vec::new()` (block all edits).
     - `Buffer::Directory(_)`: existing logic unchanged.
   - **Borrow checker**: extract cursor index and buffer type first, release borrow, then modify tasks and refresh buffer.

2. **Add `refresh_tasks_buffer` helper** in `yeet-frontend/src/update/command/task.rs`:

   ```rust
   pub fn refresh_tasks_buffer(window: &Window, contents: &mut Contents, tasks: &Tasks)
   ```

   - Walk the window tree to find `Window::Tasks(vp)`, get the `buffer_id`.
   - Rebuild lines: cancelled tasks (where `token.is_cancelled()`) use `"\x1b[9;90m{id:<4} {external_id}\x1b[0m"` (ANSI strikethrough + dim), normal tasks use plain text.

3. **Handle mode change** in `yeet-frontend/src/update/mode.rs`:
   - Navigation mode `dd` has `force: Some(Mode::Normal)` which triggers `ChangeMode`. The `to: Normal` arm uses `if let Buffer::Directory` guard — `Buffer::Tasks` already skips `yeet_buffer::update` (no change needed).

4. **Add tests**:
   - `dd` on task buffer with 3 tasks, cursor at index 1: task at index 1 has `token.is_cancelled()`, buffer line has ANSI escape codes.
   - `dd` on empty task buffer: no panic.
   - Non-`DeleteLine` modifications on task buffer are blocked.
   - `refresh_tasks_buffer` applies ANSI styling for cancelled tokens.

5. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// Normal task line
"1    rg foo"

// Cancelled task line (ANSI strikethrough + dim)
"\x1b[9;90m1    rg foo\x1b[0m"
```

## Notes

- The borrow checker is the main challenge in `modify.rs` — extract what you need from the focused buffer before mutating tasks.
- Cancellation state is derived from `CancellationToken::is_cancelled()` rather than a separate `cancelled` field — this avoids redundant state and also covers tasks cancelled via `:delt`.
- The cancelled line stays in the buffer until `TaskEnded` removes it (Prompt 7). This is intentional — it provides visual feedback that the cancel was registered.

---

# Prompt 7: Live-update the task buffer on `TaskStarted` / `TaskEnded`

**Goal**: The task window automatically refreshes when tasks start or end.

**State**: `planned`

**Motivation**: Without live updates, the task window becomes stale. Users expect to see new tasks appear and completed/cancelled tasks disappear in real time.

## Requirements

- `task::add()` refreshes the task buffer after registering a new task.
- `task::remove()` refreshes the task buffer after removing a completed task.
- Cursor is clamped if tasks are removed and the cursor was past the end.
- If no task window exists, add/remove operations do not panic or cause buffer changes.

## Exclusions

- Do NOT change the `:topen` command behavior.
- Do NOT add `:q` / `:qa` / `:tclose` logic — those are Prompts 8 and 9.

## Context

- @yeet-frontend/src/update/task.rs — `add()`, `remove()`.
- @yeet-frontend/src/update/mod.rs — `Message::TaskStarted`, `Message::TaskEnded` handlers.
- @yeet-frontend/src/model/mod.rs — `App`, `Contents`, `Window`, `Tasks`.
- The `refresh_tasks_buffer` helper from Prompt 6.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Modify `add()`** in `yeet-frontend/src/update/task.rs`:
   - Accept `&mut Contents` and `&Window` (or `&mut App`). After registering the task, call `refresh_tasks_buffer(...)`.

2. **Modify `remove()`** similarly:
   - After removing the task, call `refresh_tasks_buffer(...)`.

3. **Update dispatch** in `yeet-frontend/src/update/mod.rs`:
   - Update `Message::TaskStarted` handler to pass the additional arguments.
   - Update `Message::TaskEnded` handler similarly.

4. **Cursor clamping** in `refresh_tasks_buffer`:
   - Clamp cursor if tasks were removed and cursor was at the end.

5. **Add tests**:
   - Add a task while task window is open → buffer gains a line.
   - Remove a task while task window is open → buffer loses a line and cancelled task is gone.
   - Cursor clamping when removing the last task while cursor is on it.
   - Add/remove when no task window exists → no panic, no buffer changes.

6. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```
// Before: 2 tasks running
1    rg foo
12   fd bar

// Task "rg foo" ends → buffer refreshes
12   fd bar

// New task starts → buffer refreshes
12   fd bar
13   grep baz
```

## Notes

- The function signatures of `add()` and `remove()` change — update all call sites.
- Cursor clamping prevents panics when the user had the cursor on the last line and that task ends.

---

# Prompt 8: Implement `:q` to close focused window, `:qa` / `:qa!` to quit

**Goal**: `:q` closes the focused window in a split (collapsing the `Horizontal`). If only one window remains, `:q` shuts down yeet. `:qa` quits regardless of windows. `:qa!` force-quits.

**State**: `planned`

**Motivation**: Users need a way to close the task window and return to the normal single-pane view. The `:q` semantics match vim's behavior — close current window, or quit if it's the last one.

## Requirements

- `:q` on a `Horizontal` split: close the focused child, collapse to the remaining child, clean up closed window's buffers.
- `:q` on a single window: emit `Quit(FailOnRunningTasks)` (existing behavior).
- `:qa` always emits `Quit(FailOnRunningTasks)` regardless of window count.
- `:qa!` always emits `Quit(Force)`.
- `:q!` force-closes focused window if split, otherwise force quits.
- `Window` has a `Default` impl for `std::mem::take`.
- Closed window's buffers are removed from `app.contents.buffers` (without removing buffers still referenced by the kept window).

## Exclusions

- Do NOT add `:tclose` — that is Prompt 9.
- Do NOT change any other existing command behavior.

## Context

- @yeet-frontend/src/update/command/mod.rs — `execute` function, existing `("q", "")` and `("q!", "")` arms.
- @yeet-frontend/src/model/mod.rs — `App`, `Window`, `Contents`, `SplitFocus`.
- @yeet-frontend/src/action.rs — `Action`, `action::emit_keymap`.
- @yeet-keymap/src/message.rs — `KeymapMessage::Quit(QuitMode)`, `QuitMode`.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Add `Default` for `Window`** in `yeet-frontend/src/model/mod.rs`:

   ```rust
   impl Default for Window {
       fn default() -> Self {
           Window::Directory(Default::default(), Default::default(), Default::default())
       }
   }
   ```

2. **Modify `:q` dispatch** in `yeet-frontend/src/update/command/mod.rs`:
   - Check if `app.window` is `Horizontal`. If so, call `close_focused_window(app)`. Otherwise, emit `Quit(FailOnRunningTasks)`.

3. **Add `close_focused_window` helper**:

   ```rust
   fn close_focused_window(app: &mut App) -> Vec<Action> {
       let old_window = std::mem::take(&mut app.window);
       match old_window {
           Window::Horizontal { first, second, focus } => {
               let (kept, closed) = match focus {
                   SplitFocus::First => (*second, *first),
                   SplitFocus::Second => (*first, *second),
               };
               cleanup_window_buffers(&mut app.contents, &closed);
               app.window = kept;
               // Show cursor on new focused leaf
               Vec::new()
           }
           other => {
               app.window = other;
               vec![action::emit_keymap(KeymapMessage::Quit(QuitMode::FailOnRunningTasks))]
           }
       }
   }
   ```

4. **Implement `cleanup_window_buffers`**:
   - For `Window::Tasks(vp)`: remove `vp.buffer_id` from `app.contents.buffers`.
   - For `Window::Directory(p, c, pr)`: remove all three buffer_ids. Be careful not to remove buffers still referenced by the kept window — check the kept window's buffer_ids first.

5. **Add `:qa` and `:qa!`** match arms:
   - `("qa", "")` → emit `Quit(FailOnRunningTasks)`.
   - `("qa!", "")` → emit `Quit(Force)`.
   - Keep `("q!", "")` behavior: force-close focused window if split, otherwise force quit.

6. **Add tests**:
   - `:q` on `Horizontal { Directory, Tasks, focus: Second }` → collapses to `Directory`, task buffer removed.
   - `:q` on `Horizontal { Directory, Tasks, focus: First }` → collapses to `Tasks`, directory buffers removed.
   - `:q` on single `Directory` → returns `Quit` action.
   - `:qa` returns `Quit` action even when in a split.
   - `:qa!` returns `Quit(Force)`.
   - Buffer cleanup: closed window's buffers removed, kept window's buffers remain.

7. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// :q closes focused window in split
// Before: Horizontal { Directory, Tasks, focus: Second }
// After:  Directory (task buffer cleaned up)

// :q on single window quits
// Before: Directory
// After:  Quit(FailOnRunningTasks) action emitted
```

## Notes

- The `close_focused_window` helper swaps focus semantics: the *kept* window is the one that was NOT focused. This is correct — `:q` closes the current (focused) window.
- Buffer cleanup must account for shared buffer_ids (unlikely but possible in future).

---

# Prompt 9: Edge cases, polish, and safety

**Goal**: Handle remaining edge cases to make the task window feature production-quality.

**State**: `planned`

**Motivation**: Individual prompts focused on core functionality. This prompt sweeps up all the edge cases, safety checks, and convenience features that make the difference between a prototype and a robust feature.

## Requirements

- Dangerous operations (navigate, open, quickfix toggle) safely no-op when the task window is focused.
- Insert mode (`i`/`a`/`s`/`I`/`A`) is blocked on the task buffer.
- `:topen` when a split already exists with a `Tasks` child switches focus (no nested splits).
- `:tclose` command closes the task window specifically (convenience alternative to focusing task + `:q`).
- Empty task list works without panics (cursor handling on empty buffer).
- `collect_buffer_ids` (from Prompt 2) includes the task buffer (no garbage collection).
- `save::changes` returns early for `Buffer::Tasks`.

## Exclusions

- Do NOT refactor the window tree structure.
- Do NOT add new task buffer features beyond what is specified.

## Context

- @yeet-frontend/src/update/navigate.rs — `parent`, `selected`.
- @yeet-frontend/src/update/open.rs — `selected`.
- @yeet-frontend/src/update/qfix.rs — `toggle`.
- @yeet-frontend/src/update/mode.rs — `change()`, Insert mode arms.
- @yeet-frontend/src/update/command/mod.rs — command dispatch (for `:tclose`).
- @yeet-frontend/src/update/buffers.rs — `update`, `collect_buffer_ids`.
- @yeet-frontend/src/update/save.rs — `changes`.
- @yeet-frontend/src/update/command/task.rs — `open` (idempotency check).
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

1. **Prevent dangerous operations in task window**:
   - Audit `navigate::parent`, `navigate::selected`, `open::selected`, `qfix::toggle` — they call `get_focused_current_mut` and check for `Buffer::Directory`. Verify `Buffer::Tasks` causes early return.
   - In `mode::change()`, add `Buffer::Tasks(_)` to early-return arms in the `to: Insert` branch to block entering Insert mode on the task buffer.

2. **`:topen` idempotency**: if the window tree already contains a `Window::Tasks` child inside a `Horizontal`, switch focus to it. Don't create nested splits. (This should already be handled in Prompt 4 — verify and fix if needed.)

3. **Add `:tclose` command**: find and remove the `Tasks` child from the `Horizontal`, collapse the split. This is a convenience alternative to focusing the task window then pressing `:q`.

4. **Empty task list UX**: verify `:topen` with no running tasks creates a task buffer with zero lines and cursor handling doesn't panic.

5. **Buffer cleanup verification**: verify `collect_buffer_ids` from Prompt 2 correctly includes the task buffer's id so it doesn't get garbage-collected.

6. **`save::changes` safety**: verify it returns early for `Buffer::Tasks` (it checks `Buffer::Directory`).

7. **Add tests**:
   - Insert mode is blocked when task window is focused.
   - `navigate::parent` is a no-op when task window is focused.
   - `:topen` when split already exists → no nested split, focus switches.
   - `:tclose` collapses the split.
   - `dd` on empty task buffer → no panic.
   - `buffers::update` doesn't remove the task buffer.
   - `save::changes` returns empty when task window is focused.

8. **Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.**

## Examples

```rust
// :tclose when split exists
// Before: Horizontal { Directory, Tasks, focus: First }
// After:  Directory (task buffer cleaned up, focus on directory)

// :topen when split already exists
// Before: Horizontal { Directory, Tasks, focus: First }
// After:  Horizontal { Directory, Tasks, focus: Second }  // just switch focus
```

## Notes

- The audit in step 1 is important — any function that assumes `Buffer::Directory` will panic or misbehave if it encounters `Buffer::Tasks`. Grep for all `Buffer::Directory` matches and verify each one.
- `:tclose` shares cleanup logic with `:q` from Prompt 8. Reuse `close_focused_window` or `cleanup_window_buffers` where possible.

---

## Summary

| Prompt | What it adds | Program state after |
|--------|-------------|-------------------|
| 1 | Model types: `Window::Tasks`, `Buffer::Tasks`, `SplitFocus`, struct-field `Horizontal` | Compiles, no behavior change |
| 2 | All `todo!()` replaced with real `Horizontal`/`Tasks` implementations | Window tree infrastructure works end-to-end |
| 3 | `Ctrl+h/j/k/l` keybindings + focus switching logic | Focus navigation wired (inert — no splits exist yet) |
| 4 | `:topen` command creates split + task buffer | `:topen` creates the split in the model |
| 5 | Rendering for `Window::Tasks` and `Buffer::Tasks` | Task window is visible on screen |
| 6 | `dd` cancels tasks with visual feedback | Users can cancel tasks |
| 7 | Live updates on `TaskStarted`/`TaskEnded` | Task list auto-refreshes |
| 8 | `:q` closes focused window, `:qa`/`:qa!` quit app | Window management complete |
| 9 | Edge cases: blocked ops, `:tclose`, empty buffer, safety | Production-quality feature |
