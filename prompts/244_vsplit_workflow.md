# Overview

This feature adds user-facing commands and keymaps for creating horizontal and vertical splits of `Window::Directory` panes. The underlying split infrastructure (`Window::Horizontal`, `Window::Vertical`, layout, rendering, focus navigation, and close) is already fully implemented (see `prompts/243_vsplit.md`). This feature is the "last mile": wiring up `:split`, `:vsplit`, and `Ctrl+w Ctrl+s` / `Ctrl+w Ctrl+v` keybindings so the user can actually create splits.

The implementation is split into 3 sequential prompts, each leaving the program in a compilable and functional state:

1. [Prompt 1: Add `:split` and `:vsplit` commands](#prompt-1-add-split-and-vsplit-commands) — `done`
2. [Prompt 2: Add `Ctrl+w Ctrl+s` and `Ctrl+w Ctrl+v` keybindings](#prompt-2-add-ctrl-w-ctrl-s-and-ctrl-w-ctrl-v-keybindings) — `done`
3. [Prompt 3: Add `:split {path}` and `:vsplit {path}` with optional path arguments](#prompt-3-add-split-path-and-vsplit-path-with-optional-path-arguments) — `done`

---

# Prompt 1: Add `:split` and `:vsplit` commands

**Goal**: Add `:split` (horizontal split, top/bottom) and `:vsplit` (vertical split, left/right) ex-commands that duplicate the currently focused `Directory` window into a new split.

**State**: `done`

**Motivation**: The split infrastructure was added in `prompts/243_vsplit.md`, but there is no user-facing way to create a `Directory`-to-`Directory` split. Currently the only split-creating command is `:topen`, which creates a `Horizontal` split with a `Tasks` window. `:split` and `:vsplit` fill this gap, letting the user view the same (or different) directory in side-by-side or top-bottom panes.

## Requirements

- `:split` (no arguments) creates a `Window::Horizontal` split where:
  - `first` is the old (current) window.
  - `second` is a new `Window::Directory` initialized to the same path as the currently focused directory.
  - `focus` is `SplitFocus::Second` (the new pane gets focus).
- `:vsplit` (no arguments) creates a `Window::Vertical` split with the same semantics but side-by-side.
- The new `Directory` pane must have fresh buffer IDs for parent, current, and preview viewports (allocated via `app::get_next_buffer_id`). It must NOT share buffer IDs with the original pane.
- The new pane must emit `Action::Load` actions for its buffers so the directory contents are enumerated and rendered.
- The new pane's viewports must be configured with the same display settings as the defaults in `App::default()`: parent has `hide_cursor: true` and `show_border: true`; current has `line_number: Relative`, `line_number_width: 3`, `show_border: true`, `sign_column_width: 2`; preview has `hide_cursor: true`, `hide_cursor_line: true`.
- If the focused window is not a `Directory` (e.g., it's a `Tasks` pane), the command should be a no-op (do nothing, no error).
- After the split is created, the mode transitions from Command back to Navigation (matching the `:topen` pattern).
- The old focused pane's cursor must be hidden (`hide_cursor = true`) and the new pane's current viewport cursor must be shown.
- All existing tests continue to pass. New tests cover the split creation logic.
- Update `README.md` to document the new `:split` and `:vsplit` commands in the commands table.

## Exclusions

- Do NOT add keybindings — that is Prompt 2.
- Do NOT handle path arguments (`:split /some/path`) — that is Prompt 3.
- Do NOT change the `Window` enum, layout computation, rendering, or focus navigation — those are already done.
- Do NOT modify any existing command behavior.

## Context

- @yeet-frontend/src/update/command/mod.rs — `execute()` function where new commands are dispatched. Follow the `:topen` pattern: `("split", "") => add_change_mode(...)`.
- @yeet-frontend/src/update/command/task.rs — `open()` function: the reference implementation for creating a split. The new split module follows the same pattern but creates a `Window::Directory` instead of `Window::Tasks`.
- @yeet-frontend/src/model/mod.rs — `App::default()` (lines 35–68): the viewport configuration for a new `Directory` window. Copy these settings for the new pane.
- @yeet-frontend/src/update/app.rs — `get_next_buffer_id()`, `resolve_buffer()`, `get_focused_directory_buffer_ids()`: used to allocate buffers and get the current path.
- @yeet-frontend/src/update/navigate.rs — `navigate_to_path_with_selection()`: the function that loads directory contents into viewports. Use this to initialize the new pane after creating the split.
- @yeet-frontend/src/action.rs — `Action::Load`, `Action::WatchPath`: needed for the new pane's directory enumeration.
- @README.md — the commands table (line ~111) where `:split` and `:vsplit` must be documented.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Create `yeet-frontend/src/update/command/split.rs`

Create a new module for split command logic, following the same pattern as `task.rs`:

```rust
use std::mem;

use yeet_buffer::model::viewport::{LineNumber, ViewPort};

use crate::{
    action::Action,
    model::{App, Buffer, SplitFocus, Window},
    update::app,
};

/// Creates a horizontal (top/bottom) split of the currently focused directory.
/// The new directory pane becomes the second child and receives focus.
pub fn horizontal(app: &mut App) -> Vec<Action> {
    create_split(app, |old, new| Window::Horizontal {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

/// Creates a vertical (left/right) split of the currently focused directory.
/// The new directory pane becomes the second child and receives focus.
pub fn vertical(app: &mut App) -> Vec<Action> {
    create_split(app, |old, new| Window::Vertical {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

fn create_split(
    app: &mut App,
    make_split: impl FnOnce(Window, Window) -> Window,
) -> Vec<Action> {
    // Get the current directory path from the focused pane
    let (_, current_id, _) = match app::get_focused_directory_buffer_ids(&app.window) {
        Some(ids) => ids,
        None => return Vec::new(), // Not a Directory — no-op
    };

    let current_path = match app::get_buffer_path(app, current_id) {
        Some(path) => path.to_path_buf(),
        None => return Vec::new(),
    };

    // Allocate fresh buffer IDs for the new directory pane
    let parent_id = app::get_next_buffer_id(&mut app.contents);
    app.contents.buffers.insert(parent_id, Buffer::Empty);

    let new_current_id = app::get_next_buffer_id(&mut app.contents);
    app.contents.buffers.insert(new_current_id, Buffer::Empty);

    let preview_id = app::get_next_buffer_id(&mut app.contents);
    app.contents.buffers.insert(preview_id, Buffer::Empty);

    // Build the new Directory window with the same viewport settings as App::default()
    let new_directory = Window::Directory(
        ViewPort {
            buffer_id: parent_id,
            hide_cursor: true,
            show_border: true,
            ..Default::default()
        },
        ViewPort {
            buffer_id: new_current_id,
            line_number: LineNumber::Relative,
            line_number_width: 3,
            show_border: true,
            sign_column_width: 2,
            ..Default::default()
        },
        ViewPort {
            buffer_id: preview_id,
            hide_cursor: true,
            hide_cursor_line: true,
            ..Default::default()
        },
    );

    // Hide cursor on the old focused pane
    app.window.focused_viewport_mut().hide_cursor = true;

    // Replace the window tree with the new split
    let old_window = mem::take(&mut app.window);
    app.window = make_split(old_window, new_directory);

    // Navigate the new pane to the same directory as the original
    // This emits Action::Load for each buffer, triggering directory enumeration
    let actions = vec![
        crate::action::emit_keymap(
            yeet_keymap::message::KeymapMessage::NavigateToPath(current_path),
        ),
    ];

    actions
}
```

**Key design decisions:**
- Uses `NavigateToPath` to initialize the new pane. Since the new pane is now focused, `navigate_to_path_with_selection` will operate on its viewports and load the directory contents.
- Fresh `Buffer::Empty` entries are inserted for each new viewport. `NavigateToPath` will replace them with the correct `Buffer::Directory` / `Buffer::PathReference` entries via `resolve_buffer`.
- The old pane's cursor is hidden before the split so the user sees the cursor only in the new (focused) pane.

### Step 2: Register the module and add command dispatch in `command/mod.rs`

Add the module declaration and command match arms:

```rust
// At the top of command/mod.rs, add:
mod split;

// In the execute() match block, add before the catch-all:
("split", "") => {
    add_change_mode(mode_before, Mode::Navigation, split::horizontal(app))
}
("vsplit", "") => {
    add_change_mode(mode_before, Mode::Navigation, split::vertical(app))
}
```

### Step 3: Add tests

Add tests in `yeet-frontend/src/update/command/split.rs`:

```rust
#[cfg(test)]
mod test {
    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{App, Buffer, SplitFocus, Window};

    use super::*;

    #[test]
    fn horizontal_creates_horizontal_split() {
        let mut app = App::default();
        horizontal(&mut app);
        assert!(matches!(
            app.window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));
    }

    #[test]
    fn vertical_creates_vertical_split() {
        let mut app = App::default();
        vertical(&mut app);
        assert!(matches!(
            app.window,
            Window::Vertical {
                focus: SplitFocus::Second,
                ..
            }
        ));
    }

    #[test]
    fn horizontal_first_child_is_original_directory() {
        let mut app = App::default();
        horizontal(&mut app);
        match &app.window {
            Window::Horizontal { first, .. } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn vertical_second_child_is_new_directory() {
        let mut app = App::default();
        vertical(&mut app);
        match &app.window {
            Window::Vertical { second, .. } => {
                assert!(matches!(second.as_ref(), Window::Directory(_, _, _)));
            }
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn split_allocates_fresh_buffer_ids() {
        let mut app = App::default();
        let original_ids = app.window.buffer_ids();
        horizontal(&mut app);

        match &app.window {
            Window::Horizontal { second, .. } => {
                let new_ids = second.buffer_ids();
                for id in &new_ids {
                    assert!(
                        !original_ids.contains(id),
                        "new pane should not share buffer IDs with original"
                    );
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn split_returns_navigate_action() {
        let mut app = App::default();
        let actions = horizontal(&mut app);
        // Should contain at least one EmitMessages with NavigateToPath
        assert!(
            !actions.is_empty(),
            "split should return actions to load the new pane"
        );
    }

    #[test]
    fn split_noop_when_tasks_focused() {
        let mut app = App::default();
        // Replace window with a Tasks pane
        app.window = Window::Tasks(ViewPort::default());
        let actions = horizontal(&mut app);
        assert!(actions.is_empty());
        assert!(matches!(app.window, Window::Tasks(_)));
    }

    #[test]
    fn split_noop_when_tasks_focused_in_split() {
        let mut app = App::default();
        let old_window = std::mem::take(&mut app.window);
        app.window = Window::Horizontal {
            first: Box::new(old_window),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
        };
        let actions = vertical(&mut app);
        assert!(actions.is_empty());
        // Window should be unchanged
        assert!(matches!(app.window, Window::Horizontal { .. }));
    }

    #[test]
    fn split_registers_buffers_in_contents() {
        let mut app = App::default();
        let buffers_before = app.contents.buffers.len();
        horizontal(&mut app);
        // 3 new buffers for parent, current, preview
        assert_eq!(app.contents.buffers.len(), buffers_before + 3);
    }
}
```

### Step 4: Update `README.md`

Add `:split` and `:vsplit` to the commands table in `README.md`, after the `topen` row:

```markdown
| split                       | split current directory view horizontally (top/bottom). The new pane opens below with the same directory                                                                                                                   |
| vsplit                      | split current directory view vertically (left/right). The new pane opens to the right with the same directory                                                                                                                |
```

### Step 5: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

### `:split` — horizontal split of the current directory

```
Before:
+-------------------------------------------+
| parent | current       | preview           |
|        |               |                   |
+-------------------------------------------+
| /home/user                          3/10  |
+-------------------------------------------+
| :                                          |

After :split:
+-------------------------------------------+
| parent | current       | preview           |  <- original (first)
+-------------------------------------------+
| /home/user                          3/10  |
+-------------------------------------------+
| parent | current       | preview           |  <- new (second, focused)
+-------------------------------------------+
| /home/user                          3/10  |
+-------------------------------------------+
| :                                          |
```

### `:vsplit` — vertical split of the current directory

```
Before:
+-------------------------------------------+
| parent | current       | preview           |
|        |               |                   |
+-------------------------------------------+
| /home/user                          3/10  |
+-------------------------------------------+
| :                                          |

After :vsplit:
+-------------------+-------------------+
| par | curr | prev | par | curr | prev |  <- original (left) + new (right, focused)
|     |      |      |     |      |      |
+-------------------+-------------------+
| /home/user  3/10  | /home/user  3/10  |
+-------------------+-------------------+
| :                                      |
```

## Notes

- The `NavigateToPath` approach is used because it handles the full initialization flow: resolving buffers, setting cursor positions from history, loading the preview, and emitting `Action::Load` for directory enumeration. This avoids duplicating the complex buffer initialization logic.
- The new pane starts at the same directory as the original. The user can then navigate independently in each pane.
- Nesting splits is supported automatically — splitting an already-split layout wraps the entire current window tree in a new split node, just like `:topen` does.
- The `:q` command already handles closing splits (both `Horizontal` and `Vertical`) by collapsing to the kept child. No changes needed.
- Focus navigation (`Ctrl+h/j/k/l`) already handles any nesting of `Horizontal` and `Vertical` splits. No changes needed.

---

# Prompt 2: Add `Ctrl+w Ctrl+s` and `Ctrl+w Ctrl+v` keybindings

**Goal**: Add `Ctrl+w Ctrl+s` and `Ctrl+w Ctrl+v` key sequences in Navigation mode that create horizontal and vertical splits respectively, mirroring the `:split` and `:vsplit` commands from Prompt 1.

**State**: `done`

**Motivation**: Power users expect vim-style `Ctrl+w` split keybindings in addition to ex-commands. `Ctrl+w Ctrl+s` (horizontal split) and `Ctrl+w Ctrl+v` (vertical split) provide a fast, keyboard-driven workflow without needing to type `:split` or `:vsplit`. Using the Ctrl modifier on both keys makes it easy to hold Ctrl and press `w` then `s`/`v` in quick succession.

## Requirements

- `Ctrl+w Ctrl+s` in Navigation mode creates a horizontal split (same as `:split`).
- `Ctrl+w Ctrl+v` in Navigation mode creates a vertical split (same as `:vsplit`).
- Both keybindings use `ExecuteCommandString` to delegate to the existing `:split` / `:vsplit` commands, keeping the logic in one place.
- Both keybindings are non-repeatable (`repeatable: false`).
- Both keybindings only work in Navigation mode (not Normal, Insert, or Command modes).
- All existing keybinding tests continue to pass. New tests verify the key sequences resolve correctly.
- Update `README.md` to document the new keybindings in the navigation mode shortcuts table.

## Exclusions

- Do NOT change the split logic — that was done in Prompt 1.
- Do NOT add keybindings for modes other than Navigation.
- Do NOT add `Ctrl+w` prefix bindings for other actions (like `Ctrl+w q` for close) — those can be added separately.

## Context

- @yeet-keymap/src/map.rs — `KeyMap::default()`: where all keybindings are defined. Add the new mappings to the `Mode::Navigation` section. Follow the existing pattern for multi-key sequences (e.g., `g h` for NavigateToPath(home), `y p` for YankPathToClipboard).
- @yeet-keymap/src/message.rs — `KeymapMessage::ExecuteCommandString`: used to delegate keybindings to ex-commands. Already used for `Ctrl+n` → `cn` and `Ctrl+p` → `cN`.
- @yeet-keymap/src/key.rs — `Key`, `KeyCode`, `KeyModifier`: types for defining key sequences.
- @yeet-keymap/tests/lib_tests.rs — integration tests for keymap resolution.
- @README.md — the navigation mode shortcuts table (line ~40) where the new keybindings must be documented.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Add keybindings in `yeet-keymap/src/map.rs`

In the `Mode::Navigation` `add_mapping` call, add two new entries. Both keys in the sequence use the `Ctrl` modifier:

```rust
(
    vec![
        Key::new(KeyCode::from_char('w'), vec![KeyModifier::Ctrl]),
        Key::new(KeyCode::from_char('s'), vec![KeyModifier::Ctrl]),
    ],
    Binding {
        kind: BindingKind::Message(KeymapMessage::ExecuteCommandString(
            "split".to_owned(),
        )),
        repeatable: false,
        ..Default::default()
    },
),
(
    vec![
        Key::new(KeyCode::from_char('w'), vec![KeyModifier::Ctrl]),
        Key::new(KeyCode::from_char('v'), vec![KeyModifier::Ctrl]),
    ],
    Binding {
        kind: BindingKind::Message(KeymapMessage::ExecuteCommandString(
            "vsplit".to_owned(),
        )),
        repeatable: false,
        ..Default::default()
    },
),
```

### Step 2: Add keymap resolution tests

Add tests in `yeet-keymap/tests/lib_tests.rs` (or the appropriate test file) to verify the key sequences resolve to the correct messages:

```rust
#[test]
fn ctrl_w_ctrl_s_resolves_to_split_command() {
    let tree = KeyMap::default().into_tree();
    let mut resolver = MessageResolver::new(tree);

    resolver.add_key(&Mode::Navigation, Key::new(KeyCode::from_char('w'), vec![KeyModifier::Ctrl]));
    let (messages, _) = resolver.add_key(&Mode::Navigation, Key::new(KeyCode::from_char('s'), vec![KeyModifier::Ctrl]));

    assert_eq!(
        messages,
        vec![KeymapMessage::ExecuteCommandString("split".to_owned())]
    );
}

#[test]
fn ctrl_w_ctrl_v_resolves_to_vsplit_command() {
    let tree = KeyMap::default().into_tree();
    let mut resolver = MessageResolver::new(tree);

    resolver.add_key(&Mode::Navigation, Key::new(KeyCode::from_char('w'), vec![KeyModifier::Ctrl]));
    let (messages, _) = resolver.add_key(&Mode::Navigation, Key::new(KeyCode::from_char('v'), vec![KeyModifier::Ctrl]));

    assert_eq!(
        messages,
        vec![KeymapMessage::ExecuteCommandString("vsplit".to_owned())]
    );
}
```

### Step 3: Update `README.md`

Add the new keybindings to the navigation mode shortcuts table, after the existing `C-h/j/k/l` row:

```markdown
| C-w C-s   | split current directory view horizontally (top/bottom)      |
| C-w C-v   | split current directory view vertically (left/right)        |
```

### Step 4: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

### Key sequence: `Ctrl+w Ctrl+s`

```
User presses Ctrl+w:
  KeySequence::Changed("^W")   <- key buffer shows pending sequence

User presses Ctrl+s:
  KeySequence::Completed       <- resolved to ExecuteCommandString("split")
  → command::execute("split")  <- creates horizontal split
```

### Key sequence: `Ctrl+w Ctrl+v`

```
User presses Ctrl+w:
  KeySequence::Changed("^W")   <- key buffer shows pending sequence

User presses Ctrl+v:
  KeySequence::Completed       <- resolved to ExecuteCommandString("vsplit")
  → command::execute("vsplit") <- creates vertical split
```

## Notes

- Using `ExecuteCommandString` instead of a new `KeymapMessage` variant keeps the split logic centralized in the command module. This avoids duplicating the split creation code and ensures `:split`/`:vsplit` and `Ctrl+w Ctrl+s`/`Ctrl+w Ctrl+v` always behave identically.
- The `Ctrl+w` prefix is a natural vim-style window management prefix. It does not conflict with any existing bindings — `Ctrl+w` is currently unbound.
- Both keys in the sequence use the `Ctrl` modifier. This allows the user to hold `Ctrl` throughout the sequence (`Ctrl+w` → `Ctrl+s`) without lifting the modifier key, which is ergonomically faster.
- These bindings are Navigation-mode only because splits are a navigation-level concept. In Normal mode, `Ctrl+w` could conflict with word-deletion semantics in text editing.

---

# Prompt 3: Add `:split {path}` and `:vsplit {path}` with optional path arguments

**Goal**: Extend `:split` and `:vsplit` to accept an optional directory path argument, so `:split /tmp` opens a horizontal split with the new pane navigated to `/tmp` instead of the current directory.

**State**: `done`

**Motivation**: Without path arguments, the user must first split, then navigate the new pane to the desired directory. Supporting `:split /tmp` streamlines this workflow, matching vim's `:split {file}` pattern adapted for a file manager context.

## Requirements

- `:split {path}` creates a horizontal split with the new pane navigated to `{path}`.
- `:vsplit {path}` creates a vertical split with the new pane navigated to `{path}`.
- `:split` and `:vsplit` without arguments continue to work as before (Prompt 1), opening the current directory.
- If `{path}` is a file, navigate to its parent directory and select the file (matching `navigate::path` behavior).
- If `{path}` does not exist, emit an error message and do not create a split.
- If `{path}` is relative, resolve it relative to the current directory of the focused pane.
- All existing tests continue to pass. New tests cover path argument parsing and edge cases.
- Update `README.md` to reflect that `:split` and `:vsplit` accept an optional `{path}` argument in the commands table.

## Exclusions

- Do NOT change keybindings — `Ctrl+w Ctrl+s`/`Ctrl+w Ctrl+v` remain argument-less.
- Do NOT add tab-completion or path auto-complete for the command arguments — that is a separate feature.
- Do NOT handle multiple paths or glob patterns.

## Context

- @yeet-frontend/src/update/command/mod.rs — `execute()`: the command dispatch. Currently `:split` matches `("split", "")`. Change to also match `("split", args)` when `args` is non-empty.
- @yeet-frontend/src/update/command/split.rs — the split module from Prompt 1. Extend `horizontal()` and `vertical()` to accept an optional `target_path` parameter.
- @yeet-frontend/src/update/navigate.rs — `navigate_to_path_with_selection()`: handles file-vs-directory path resolution. The split functions should use `NavigateToPath` which delegates to this function.
- @yeet-frontend/src/update/app.rs — `get_buffer_path()`: get the current pane's directory path for resolving relative paths.
- @README.md — the commands table where `:split` and `:vsplit` rows (added in Prompt 1) must be updated to show the optional `{path}` argument.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Extend `split.rs` to accept an optional target path

Modify the `horizontal` and `vertical` functions to accept an optional path. Add a `horizontal_to` and `vertical_to` variant, or add an `Option<PathBuf>` parameter:

```rust
pub fn horizontal(app: &mut App, target: Option<PathBuf>) -> Vec<Action> {
    create_split(app, target, |old, new| Window::Horizontal {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

pub fn vertical(app: &mut App, target: Option<PathBuf>) -> Vec<Action> {
    create_split(app, target, |old, new| Window::Vertical {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

fn create_split(
    app: &mut App,
    target: Option<PathBuf>,
    make_split: impl FnOnce(Window, Window) -> Window,
) -> Vec<Action> {
    // Get the current path (used as default target and for relative path resolution)
    let (_, current_id, _) = match app::get_focused_directory_buffer_ids(&app.window) {
        Some(ids) => ids,
        None => return Vec::new(),
    };

    let current_path = match app::get_buffer_path(app, current_id) {
        Some(path) => path.to_path_buf(),
        None => return Vec::new(),
    };

    // Determine the navigation target
    let navigate_to = match target {
        Some(path) => {
            // Resolve relative paths against the current directory
            let resolved = if path.is_relative() {
                current_path.join(&path)
            } else {
                path
            };
            resolved
        }
        None => current_path,
    };

    // ... rest of create_split unchanged, using navigate_to instead of current_path
    // for the NavigateToPath action
}
```

### Step 2: Update command dispatch in `command/mod.rs`

Change the match arms from exact empty-args matches to also handle non-empty args:

```rust
("split", "") => {
    add_change_mode(mode_before, Mode::Navigation, split::horizontal(app, None))
}
("split", args) => {
    let path = PathBuf::from(args.trim());
    add_change_mode(mode_before, Mode::Navigation, split::horizontal(app, Some(path)))
}
("vsplit", "") => {
    add_change_mode(mode_before, Mode::Navigation, split::vertical(app, None))
}
("vsplit", args) => {
    let path = PathBuf::from(args.trim());
    add_change_mode(mode_before, Mode::Navigation, split::vertical(app, Some(path)))
}
```

**Note:** Since the match arms with `""` come first, the no-argument case is handled by the first arm, and only non-empty args fall through to the second arm. The `args` variants trim whitespace before constructing the path.

### Step 3: Handle non-existent paths

In `create_split`, add a check after path resolution:

```rust
let navigate_to = match target {
    Some(path) => {
        let resolved = if path.is_relative() {
            current_path.join(&path)
        } else {
            path
        };
        if !resolved.exists() {
            return vec![Action::EmitMessages(vec![Message::Error(
                format!("Path does not exist: {}", resolved.display()),
            )])];
        }
        resolved
    }
    None => current_path,
};
```

### Step 4: Add tests

```rust
#[test]
fn horizontal_with_path_navigates_to_target() {
    let mut app = App::default();
    let target = std::env::temp_dir();
    let actions = horizontal(&mut app, Some(target.clone()));

    assert!(matches!(
        app.window,
        Window::Horizontal { focus: SplitFocus::Second, .. }
    ));
    // Actions should include a NavigateToPath pointing to the target
    assert!(!actions.is_empty());
}

#[test]
fn split_with_nonexistent_path_returns_error() {
    let mut app = App::default();
    let actions = horizontal(&mut app, Some(PathBuf::from("/nonexistent/path/12345")));
    // Should NOT create a split
    assert!(matches!(app.window, Window::Directory(_, _, _)));
    // Should return an error action
    assert!(!actions.is_empty());
}
```

### Step 5: Update Prompt 1 tests

Adjust the `horizontal()` and `vertical()` calls in Prompt 1 tests to pass `None` as the target path.

### Step 6: Update `README.md`

Update the `:split` and `:vsplit` rows in the commands table to reflect the optional path argument:

```markdown
| split \<path>               | split current directory view horizontally (top/bottom). The new pane opens below showing \<path>, or the current directory if no path is given. Path can be absolute or relative to the current directory                   |
| vsplit \<path>              | split current directory view vertically (left/right). The new pane opens to the right showing \<path>, or the current directory if no path is given. Path can be absolute or relative to the current directory                |
```

### Step 7: Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`.

## Examples

### `:split /tmp`

```
Before:
+-------------------------------------------+
| parent | current       | preview           |
|        | /home/user    |                   |
+-------------------------------------------+

After :split /tmp:
+-------------------------------------------+
| parent | current       | preview           |  <- original, showing /home/user
+-------------------------------------------+
| parent | current       | preview           |  <- new, showing /tmp (focused)
+-------------------------------------------+
```

### `:vsplit ../` (relative path)

```
Before:
+-------------------------------------------+
| parent | current       | preview           |
|        | /home/user    |                   |
+-------------------------------------------+

After :vsplit ../:
+-------------------+-------------------+
| par | curr | prev | par | curr | prev |
|     | /home/user   |     | /home       |  <- new pane navigated to parent
+-------------------+-------------------+
```

### `:split /nonexistent`

```
Error message displayed: "Path does not exist: /nonexistent"
No split is created.
```

## Notes

- The `NavigateToPath` message handles file-vs-directory resolution automatically via `navigate::path`. If `:vsplit /home/user/file.txt` is given, the new pane navigates to `/home/user/` with `file.txt` selected. This is free because `NavigateToPath` already has this logic.
- Relative paths are resolved against the currently focused directory's path, not the working directory of the yeet process. This matches user expectations — when looking at `/home/user`, `:split ../` means `/home/`.
- The path existence check prevents creating an empty split that would confuse the user. The error message follows the existing pattern used by other commands (e.g., `cp`, `mv`).
- This prompt changes the function signatures from Prompt 1 (`horizontal(app)` → `horizontal(app, target)`). All call sites (commands and tests) must be updated. The keybindings from Prompt 2 use `ExecuteCommandString("split")` which goes through the command dispatch, so they automatically get the no-argument behavior.
