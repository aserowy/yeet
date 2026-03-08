# Overview

This feature adds Vim-like tabs to yeet, including a tab bar, commands for creating/closing/navigating tabs, Navigation-mode keymaps for `gt`/`gT`, and a `:tabs` listing. The implementation is split into sequential prompts, each leaving the program in a runnable and functional state.

1. [Prompt 1: Tab data model scaffolding](#prompt-1-tab-data-model-scaffolding) — `done`
2. [Prompt 2: Current-tab window plumbing](#prompt-2-current-tab-window-plumbing) — `done`
3. [Prompt 3: Tab bar rendering + layout offset](#prompt-3-tab-bar-rendering--layout-offset) — `done`
4. [Prompt 4: Tab commands — create/close/switch](#prompt-4-tab-commands--createcloseswitch) — `done`
5. [Prompt 5: Navigation keymaps `gt`/`gT`](#prompt-5-navigation-keymaps-gtgt) — `done`
6. [Prompt 6: `:tabs` command output](#prompt-6-tabs-command-output) — `done`
7. [Prompt 7: Guard tab close on unsaved buffers](#prompt-7-guard-tab-close-on-unsaved-buffers) — `planned`

---

# Prompt 1: Tab data model scaffolding

**Goal**: Introduce tab state in the frontend model and remove the legacy `app.window` field.

**State**: `done`

**Motivation**: Tabs are a top-level navigation concept like in Vim. We need a durable model foundation before routing layout, rendering, and commands through it.

## Requirements

- Add tab storage to `App` as `HashMap<usize, Window>` and introduce `current_tab_id: usize` alongside it.
- Remove the `window` field from `App` entirely in this prompt.
- Initialize the first tab at startup (the existing initial window should live in tab id 1). There is always at least one tab while the app is running.
- Add a compatibility accessor for the active window (e.g., `app.current_window()` and `app.current_window_mut()`), but do not yet replace all call sites.
- Add tests that validate default App initializes with a tab map containing one window and `current_tab_id` set.

## Exclusions

- Do **not** route layout or rendering through tabs yet.
- Do **not** implement tab commands (`:tabnew`, `:tabc`, etc.) or keymaps (`gt`, `gT`) in this prompt.
- Do **not** change existing commandline behavior or quickfix/task handling.
- Do **not** introduce any new async tasks.

## Context

- App model and Window definitions: @yeet-frontend/src/model/mod.rs
- Update loop that calls window update: @yeet-frontend/src/update/mod.rs
- Project conventions and testing rules: @AGENTS.md

## Implementation Plan

1. **Model struct changes**: update `App` to include `tabs: HashMap<usize, Window>` and `current_tab_id: usize`.
2. **Remove legacy field**: delete the `window` field from `App` and update any constructors/tests that still initialize it.
3. **Default initialization**:
   - Build the initial window with `Window::create(...)` as before.
   - Insert it into a `HashMap` with id `1`.
   - Set `current_tab_id = 1`.
4. **Accessors**:
   - Add `current_window()` and `current_window_mut()` that read from `tabs` via `current_tab_id`.
   - Return `Result<&Window, AppError>` / `Result<&mut Window, AppError>` instead of `expect(...)`.
5. **Tests**:
   - Add a unit test that asserts `App::default()` has `current_tab_id == 1` and `tabs.len() == 1`.
   - Assert that `tabs[&1]` is the default `Window` type (e.g., `Window::Directory`).

```rust
impl Default for App {
    fn default() -> Self {
        let mut buffers = HashMap::new();
        buffers.insert(1, Buffer::Empty);
        let window = Window::create(1, 1, 1);
        let mut tabs = HashMap::new();
        tabs.insert(1, window);

        Self {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 1,
            },
            tabs,
            current_tab_id: 1,
        }
    }
}
```

```rust
pub struct App {
    pub commandline: CommandLine,
    pub contents: Contents,
    pub tabs: HashMap<usize, Window>,
    pub current_tab_id: usize,
}

impl App {
    pub fn current_window(&self) -> Result<&Window, AppError> {
        self.tabs
            .get(&self.current_tab_id)
            .ok_or_else(|| AppError::InvalidState("current_tab_id missing from tabs".to_string()))
    }

    pub fn current_window_mut(&mut self) -> Result<&mut Window, AppError> {
        self.tabs
            .get_mut(&self.current_tab_id)
            .ok_or_else(|| AppError::InvalidState("current_tab_id missing from tabs".to_string()))
    }
}
```

## Examples

```rust
let app = App::default();
assert_eq!(app.current_tab_id, 1);
assert!(app.tabs.contains_key(&1));
```

## Notes

- Removing `app.window` means later prompts must route through `current_window()` exclusively.

---

# Prompt 2: Current-tab window plumbing

**Goal**: Route layout and rendering through the current tab’s window, without adding new commands or keymaps yet.

**State**: `done`

**Motivation**: After introducing tabs, the rest of the app should read from the current tab’s window to preserve behavior while enabling new tab features.

## Requirements

- All rendering and layout must use `app.current_window()` (no `app.window` remains).
- Ensure `window::update` sets viewports for the current tab’s `Window`.
- Ensure buffer rendering and statuslines are based on the current tab’s `Window`.
- Update or add tests to cover window update calls routed through the current tab.

## Exclusions

- Do not implement any tab commands or keymaps yet.
- Do not render the tab bar yet.

## Context

- App model and Window definitions: @yeet-frontend/src/model/mod.rs
- Window sizing/layout: @yeet-frontend/src/update/window.rs
- Window rendering entrypoint: @yeet-frontend/src/view/window.rs
- Buffer rendering + statusline: @yeet-frontend/src/view/buffer.rs, @yeet-frontend/src/view/statusline.rs
- Update loop that calls window update: @yeet-frontend/src/update/mod.rs

## Implementation Plan

1. **Layout routing**: update `update/window.rs` so `set_buffer_vp(...)` is called with `app.current_window_mut()`.
2. **Render routing**:
   - In `view/window.rs`, use `app.current_window()` to compute height and pass into buffer rendering.
   - In `view/buffer.rs`, start traversal from `app.current_window()` instead of any legacy field.
3. **Buffer/statusline consistency**: ensure statusline rendering still uses the same viewport/buffer pairs, now reached through the current tab window.
4. **Tests**:
   - Update any tests constructing `App` to use `tabs` and `current_tab_id`.
   - Add a test that mutates the current tab window and confirms layout updates affect that tab.

```rust
pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    buffer::view(&model.state.modes.current, &model.app, frame, 0, 0);
    model.app.current_window().get_height()
}
```

```rust
pub fn update(app: &mut App, area: Rect) -> Result<(), AppError> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(u16::try_from(app.commandline.buffer.lines.len())?),
        ])
        .split(area);

    set_buffer_vp(app.current_window_mut(), main[0])?;
    set_commandline_vp(&mut app.commandline, main[1])?;
    Ok(())
}
```

## Examples

```rust
buffer::view(&model.state.modes.current, &model.app, frame, 0, 0);
// uses model.app.current_window() internally
```

## Notes

- Keep the rest of the update pipeline unchanged; only swap the window source.

---

# Prompt 3: Tab bar rendering + layout offset

**Goal**: Render a tab bar above all windows when more than one tab exists and reserve layout space for it.

**State**: `done`

**Motivation**: The tab bar is the visual anchor for tabs; it must integrate cleanly with window layout and statuslines.

## Requirements

- Render a tab bar **above all windows** when more than one tab exists. Do not render the bar for a single tab.
- Tab titles must be derived from the **focused window** of each tab. If the focused buffer is a directory, show only the **current folder name** (not full path). Use the existing statusline label styling as inspiration.
- Ensure view/layout calculations reserve one row for the tab bar when it is visible, so windows and statuslines still fit.
- Add tests that validate layout offset/height handling does not break `Window::get_height` logic.

## Exclusions

- Do not implement tab commands or keymaps in this prompt.

## Context

- App model and Window definitions: @yeet-frontend/src/model/mod.rs
- Window sizing/layout: @yeet-frontend/src/update/window.rs
- Window rendering entrypoint: @yeet-frontend/src/view/window.rs
- Buffer rendering + statusline: @yeet-frontend/src/view/buffer.rs, @yeet-frontend/src/view/statusline.rs

## Implementation Plan

1. **Tab-bar module**:
   - Add a `view/tabbar.rs` (or extend `view/window.rs`) with a `render_tabbar(...)` helper.
   - Only render when `app.tabs.len() > 1`.
2. **Tab title derivation**:
   - For each tab id, get the focused viewport and resolve its buffer.
   - If directory-focused, extract only the final folder name (no full path).
   - Provide a fallback label like `(empty)` when no path exists.
3. **Layout offset**:
   - In `view/window::view`, render the tab bar at `y=0` and offset buffer rendering by `+1` row.
   - In `update/window::update`, subtract 1 row from the main buffer area when the bar is visible.
4. **Styling**:
   - Use statusline colors: active tab bold/white, inactive tabs gray.
   - Separate tabs with a delimiter (e.g., ` | `) without overflowing width.
5. **Tests**:
   - Add a layout test that ensures the current tab window’s viewport `y` is offset by 1 when multiple tabs exist.
   - Add a label derivation test for directory focus → folder name only.

```rust
fn tab_title_from_window(window: &Window, buffers: &HashMap<usize, Buffer>) -> String {
    let focused = window.focused_viewport();
    match buffers.get(&focused.buffer_id) {
        Some(Buffer::Directory(dir)) => dir
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("(empty)")
            .to_string(),
        _ => "(empty)".to_string(),
    }
}
```

```rust
let show_tabs = app.tabs.len() > 1;
let vertical_offset = if show_tabs { 1 } else { 0 };
buffer::view(&model.state.modes.current, &model.app, frame, 0, vertical_offset);
```

## Examples

- With three tabs, the top row shows: `1: src | 2: tests | 3: (empty)`.
- With one tab, no tab bar is rendered and the window occupies the full height.

## Notes

- Tab title formatting should align with statusline label styling.

---

# Prompt 4: Tab commands — create/close/switch

**Goal**: Implement Vim-like tab commands for creating, closing, and switching tabs.

**State**: `done`

**Motivation**: Users need command-driven control over tabs, matching Vim behavior.

## Requirements

- Implement the following commands with Vim defaults:
  - `:tabnew` — create a new tab. Use `Window::create(...)` but **navigate to the currently focused directory** in the current tab. If the focused window is `Tasks`/`topen`, navigate to the home directory instead. The new tab becomes current.
  - `:tabc` — close the current tab. If it is the last remaining tab, quit the app (same behavior as Vim).
  - `:tabo` — close all other tabs, keeping the current one.
  - `:tabfir` — go to first tab.
  - `:tabl` — go to last tab.
  - `:tabn` — go to next tab (wrap to first at end).
  - `:tabp` — go to previous tab (wrap to last at start).
- Update or add tests for `tabnew`, `tabc`, and `tabn/tabp` wrapping behavior.

## Exclusions

- Do not add `gt/gT` keymaps in this prompt.
- Do not implement `:tabs`.

## Context

- Command dispatch: @yeet-frontend/src/update/command/mod.rs
- App/window helpers: @yeet-frontend/src/update/app.rs
- Model definition (tabs, current tab): @yeet-frontend/src/model/mod.rs
- Home path helper: @yeet-keymap/src/map.rs (see `get_home_path()` usage)

## Implementation Plan

1. **Helper module**: add `update/tab.rs` with functions like `create_tab`, `close_tab`, `close_other_tabs`, `first_tab`, `last_tab`, `next_tab`, `previous_tab`.
2. **Tab creation**:
   - Determine the current focused directory path (or fallback to home if focused buffer is Tasks).
   - Create a new `Window::create(...)` and then navigate it to that path.
   - Insert into `tabs` with a fresh id and set `current_tab_id` to the new id.
3. **Tab closing**:
   - On `:tabc`, if only one tab remains, emit `Quit` (Vim-like).
   - Otherwise remove the current tab and select the next or previous id (prefer next).
4. **Tab switching**:
   - Implement wraparound for `tabn/tabp` using ordered tab ids.
   - `tabfir` and `tabl` pick first/last id in sorted order.
5. **Command wiring**:
   - Add match arms in `command::execute` for `tabnew`, `tabc`, `tabo`, `tabfir`, `tabl`, `tabn`, `tabp`.
   - Use `add_change_mode` for consistent mode transitions.
6. **Tests**:
   - `tabnew` creates a new tab and focuses it.
   - `tabc` on last tab emits `Quit`.
   - `tabn/tabp` wrap correctly across multiple tabs.

```rust
pub fn close_tab(app: &mut App) -> Option<QuitMode> {
    if app.tabs.len() == 1 {
        return Some(QuitMode::FailOnRunningTasks);
    }
    let ordered = ordered_tab_ids(app);
    let next = next_tab_id(app.current_tab_id, &ordered);
    app.tabs.remove(&app.current_tab_id);
    app.current_tab_id = next;
    None
}
```

```rust
pub fn next_tab_id(current: usize, ordered: &[usize]) -> usize {
    let pos = ordered.iter().position(|id| *id == current).unwrap_or(0);
    ordered[(pos + 1) % ordered.len()]
}
```

## Examples

- `:tabn` on last tab wraps to first.
- `:tabp` on first wraps to last.
- `:tabc` on the only tab quits the app.

## Notes

- Keep tab ids stable across navigations; when closing a tab, prefer next, otherwise previous.

---

# Prompt 5: Navigation keymaps `gt`/`gT`

**Goal**: Add Navigation-mode keymaps for `gt` (next tab) and `gT` (previous tab).

**State**: `done`

**Motivation**: Vim users expect `gt/gT` for tab navigation.

## Requirements

- Add Navigation-only bindings:
  - `gt` → next tab (wrap to first).
  - `gT` → previous tab (wrap to last).
- Add or update keymap tests to validate bindings in Navigation mode only.

## Exclusions

- Do not add new commands in this prompt.

## Context

- Keymap bindings: @yeet-keymap/src/map.rs
- Keymap tests: @yeet-keymap/tests/lib_tests.rs

## Implementation Plan

1. **Bindings**:
   - Add `g` `t` to Navigation mode mapping → `ExecuteCommandString("tabn")`.
   - Add `g` `T` to Navigation mode mapping → `ExecuteCommandString("tabp")`.
2. **Mode scope**: ensure no `gt/gT` mappings are added to Navigation mode.
3. **Tests**:
   - Add keymap tests asserting Navigation mode resolves `gt` and `gT` to the expected messages.
   - Add a negative test that Normal mode does not resolve `gt/gT` (if patterns exist).

```rust
(
    vec![
        Key::new(KeyCode::from_char('g'), vec![]),
        Key::new(KeyCode::from_char('T'), vec![]),
    ],
    Binding {
        kind: BindingKind::Message(KeymapMessage::ExecuteCommandString("tabp".to_owned())),
        ..Default::default()
    },
),
```

```rust
(
    vec![Key::new(KeyCode::from_char('g'), vec![]), Key::new(KeyCode::from_char('t'), vec![])],
    Binding {
        kind: BindingKind::Message(KeymapMessage::ExecuteCommandString("tabn".to_owned())),
        ..Default::default()
    },
),
```

## Examples

- In Navigation mode, pressing `g` then `t` switches to the next tab.

## Notes

- Ensure `gt/gT` are only mapped in Navigation mode (not Normal).

---

# Prompt 6: `:tabs` command output

**Goal**: Implement a Vim-like `:tabs` output listing, inspired by `:cl` formatting.

**State**: `done`

**Motivation**: Users need a quick overview of all tabs and the current selection.

## Requirements

- Implement `:tabs` output using a `print::tabs(...)` helper.
- Output format should be Vim-like and modeled on `:cl`:
  - `:tabs`
  - `> 1  /absolut/path/src`
  - `  2  /absolut/path/tests`
  - `  3  (empty)`
  - `  4  Tasks`
- Include a marker for the current tab.
- Add tests to validate output ordering and current marker.

## Exclusions

## Context

- Command printing style: @yeet-frontend/src/update/command/print.rs
- Command dispatch: @yeet-frontend/src/update/command/mod.rs
- Model tabs/current tab: @yeet-frontend/src/model/mod.rs

## Implementation Plan

1. **Print helper**:
   - Add `print::tabs(...)` in `update/command/print.rs` modeled on `print::qfix`.
   - Build an ordered list of tab ids and titles; mark current tab with `>`.
2. **Command dispatch**:
   - Add `("tabs", "")` arm in `command::execute` to call `print::tabs`.
3. **Output formatting**:
   - Match `:cl` style: header line `:tabs` followed by aligned tab rows.
   - Ensure titles use the same label derivation as the tab bar.
4. **Tests**:
   - Verify ordering and current marker.
   - Verify `(empty)` placeholder when no directory path is available.

```rust
pub fn tabs(app: &App) -> Vec<Action> {
    let mut lines = vec![":tabs".to_string()];
    let ordered = ordered_tab_ids(app);
    for id in ordered {
        let title = tab_title_for_id(app, id);
        let prefix = if id == app.current_tab_id { ">" } else { " " };
        lines.push(format!("{} {:<2} {}", prefix, id, title));
    }
    let content = lines
        .into_iter()
        .map(PrintContent::Default)
        .collect::<Vec<_>>();
    vec![action::emit_keymap(KeymapMessage::Print(content))]
}
```

```rust
let mut lines = vec![":tabs".to_string()];
for id in ordered_ids {
    let prefix = if id == current { ">" } else { " " };
    lines.push(format!("{} {:<2} {}", prefix, id, title));
}
```

## Examples

- `:tabs` prints all tabs with a `>` on the current one.

## Notes

- Use the same title derivation logic as the tab bar.

---

# Prompt 7: Guard tab close on unsaved buffers

**Goal**: Make tab-close commands respect unsaved buffer changes, mirroring `:q` behavior for focused buffers.

**State**: `done`

**Motivation**: Users should not lose unsaved work when closing a tab. Tab close should behave like buffer close: if any buffer shown in the tab has unsaved changes, block the close and show the same warning as `:q`.

## Requirements

- When running `:tabc` or `:tabo`, prevent closing any tab that contains **visible buffers with unsaved changes**.
- The unsaved check must mirror `:q` logic in `@yeet-frontend/src/update/command/mod.rs`:
  - Use the same error message: `No write since last change (add ! to override)`.
  - Scope the check to **buffers shown in the tab's windows** (not all buffers in the app).
- If a tab close is blocked, do **not** change the current tab or close any tabs.
- Add a force variant for tab close (`:tabc!` and `:tabo!`) that skips the unsaved check and always closes.
- On forced tab close, **roll back unsaved changes** for buffers that belong to any tab being closed (same intent as `:q!`).
- Add tests covering:
  - `:tabc` blocked when the current tab shows a dirty buffer.
  - `:tabo` blocked when any other tab shows a dirty buffer.
  - `:tabc!` and `:tabo!` ignore unsaved checks and proceed.

## Exclusions

- Do not change `:q`, `:q!`, `:qa`, or `:qa!` behavior.
- Do not alter buffer save mechanics or undo behavior.
- Do not change tab title rendering or tab listing output.

## Context

- `:q` unsaved-change handling and error messaging: @yeet-frontend/src/update/command/mod.rs
- Tab command dispatch: @yeet-frontend/src/update/command/mod.rs
- Tab helpers and close behavior: @yeet-frontend/src/update/tab.rs
- App model tabs/windows: @yeet-frontend/src/model/mod.rs
- Project conventions and tests: @AGENTS.md

## Implementation Plan

1. **Introduce shared helpers**:
   - Add a helper that checks a `Window` for unsaved changes by inspecting its `buffer_ids()` against `Contents`.
   - Add a helper that **resets undo state** for all directory buffers in a `Window` (mirror `reset_unsaved_changes` used by `:q!`).
   - Reuse the `has_unsaved_changes` logic but scope it to the buffer IDs in the target tab.
2. **Wire into command dispatch**:
   - In `command::execute`, add `tabc!` and `tabo!` command arms.
   - For `tabc` and `tabo`, call the unsaved check before closing. If dirty, emit the same error as `:q` via `print_error(...)` and return early.
3. **Close logic**:
   - For `tabo`, if any tab to be closed contains dirty buffers, block the entire operation.
   - For `tabc!` / `tabo!`, **reset undo state** for buffers in tabs being closed before removing them.
4. **Tests**:
   - Add tests in `command/mod.rs` covering blocked and forced close cases.
   - Reuse existing helpers for creating dirty directory buffers.

```rust
fn tab_has_unsaved_changes(app: &App, tab_id: usize) -> bool {
    let window = app.tabs.get(&tab_id)?;
    let ids = window.buffer_ids();
    ids.into_iter().any(|id| has_unsaved_changes(&app.contents, Some(id)))
}
```

```rust
// tabc
if tab_has_unsaved_changes(app, app.current_tab_id) {
    return print_error("No write since last change (add ! to override)", mode_before, mode);
}
```

## Examples

- `:tabc` on a tab whose focused directory has unsaved changes → error and no close.
- `:tabo` when any other tab shows an unsaved directory buffer → error and no tabs are closed.
- `:tabc!` closes the tab regardless of unsaved changes and rolls back those changes in the closed tab.
- `:tabo!` closes other tabs, rolling back unsaved changes in each closed tab.

## Notes

- Use the existing `:q` wording and error path for consistency.
- Forced tab close should mirror `:q!` intent: discard unsaved changes for closed tabs only, leaving kept tabs untouched.
- Ensure tests assert that the tab count and `current_tab_id` remain unchanged when blocked.
