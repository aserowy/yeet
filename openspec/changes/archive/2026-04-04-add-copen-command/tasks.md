## 1. Model Layer

- [x] 1.1 Add `Window::QuickFix(ViewPort)` variant to the `Window` enum in `yeet-frontend/src/model/mod.rs`
- [x] 1.2 Update all `Window` match arms: `focused_viewport`, `focused_window_mut`, `focused_viewport_mut`, `buffer_ids`, `contains_tasks` (add `contains_quickfix`), `Default` — follow the `Tasks` pattern
- [x] 1.3 Add `QuickFixBuffer` struct (with `TextBuffer` field) and `Buffer::QuickFix(QuickFixBuffer)` variant to the `Buffer` enum
- [x] 1.4 Update `Buffer::resolve_path` to return `None` for `QuickFix` variant
- [x] 1.5 Write tests for `Window::QuickFix` construction, `buffer_ids`, `contains_quickfix`, and `focused_viewport` through splits

## 2. Copen Open Command

- [x] 2.1 Create `yeet-frontend/src/update/command/qfix_window.rs` module with `open` function following `task::open` pattern
- [x] 2.2 Implement `focus_quickfix` function (mirrors `focus_tasks`) for idempotent open behavior
- [x] 2.3 Implement `build_qfix_lines` to render quickfix entries with index, path, removed status, and ANSI bold for `current_index`
- [x] 2.4 Register `:copen` command in `yeet-frontend/src/update/command/mod.rs`
- [x] 2.5 Write tests: open creates horizontal split, open with entries renders formatted lines, open with empty qfix creates empty buffer, idempotent open switches focus

## 3. Buffer Refresh

- [x] 3.1 Implement `refresh_quickfix_buffer` function (mirrors `refresh_tasks_buffer`) that rebuilds lines and adjusts cursor
- [x] 3.2 Implement `find_quickfix_viewport_mut` helper to locate QuickFix viewport in window tree
- [x] 3.3 Call `refresh_quickfix_buffer` after `:cfirst`, `:cn`, `:cN` in `command/qfix.rs`
- [x] 3.4 Call `refresh_quickfix_buffer` after `:clearcl` in `command/qfix.rs`
- [x] 3.5 Call `refresh_quickfix_buffer` after toggle and invert in `update/qfix.rs`
- [x] 3.6 Call `refresh_quickfix_buffer` after `add` in `update/qfix.rs`
- [x] 3.7 Write tests: refresh updates bold on index change, refresh after clear shows empty, refresh noop without quickfix window

## 4. Entry Removal (dd)

- [x] 4.1 Implement `remove_entry` function: remove entry at cursor index from `QuickFix.entries`, adjust `current_index` (decrement if before, clamp if at/after), remove sign from all buffers, rebuild copen buffer
- [x] 4.2 Write tests: remove before current_index decrements it, remove at current_index clamps it, remove after current_index leaves it unchanged, remove last entry sets index to 0, sign is removed on dd

## 5. Nearest Directory Window Navigation (enter)

- [x] 5.1 Implement `find_nearest_directory_viewport` that locates the split containing QuickFix, takes the sibling subtree, and follows focus path to find first Directory window
- [x] 5.2 Implement enter handler: get selected entry path from cursor position, resolve nearest directory, emit `NavigateToPathAsPreview` action
- [x] 5.3 Write tests: enter navigates sibling directory, enter navigates nested sibling via focus path, enter is noop when no directory in sibling

## 6. Keymap Routing

- [x] 6.1 Add QuickFix window type handling in the keymap layer, allowing only topen-shared navigation keys, enter, and dd
- [x] 6.2 Ensure all other keymaps are no-ops when copen is focused
- [x] 6.3 Write tests: navigation keys work in copen, unmapped keys produce no action

## 7. Statusline

- [x] 7.1 Add `quickfix_status` function in `yeet-frontend/src/view/statusline.rs` (focused: bold "QuickFix" label + position)
- [x] 7.2 Add `quickfix_status_unfocused` function (non-bold "QuickFix" label)
- [x] 7.3 Update `statusline::view` match to handle `Buffer::QuickFix`
- [x] 7.4 Write tests for statusline rendering if applicable

## 8. Integration and Cleanup

- [x] 8.1 Update `tab_title_from_window_full_path` in `print.rs` to handle `Window::QuickFix`
- [x] 8.2 Update any remaining match arms across the codebase that the compiler flags for exhaustiveness (view layer, app utilities, split handling)
- [x] 8.3 Run `cargo clippy` and `cargo fmt` and fix all warnings
- [x] 8.4 Run `cargo test` and ensure all tests pass
