## 1. Extract shared statusline functions

- [x] 1.1 Add `label_status(label: &str, line_count: usize, viewport: &ViewPort, frame: &mut Frame, rect: Rect, theme: &Theme)` function that contains the shared focused statusline logic (bold label + position indicator)
- [x] 1.2 Add `label_status_unfocused(label: &str, frame: &mut Frame, rect: Rect, theme: &Theme)` function that contains the shared unfocused statusline logic (plain label)

## 2. Update call sites and remove duplicates

- [x] 2.1 Update the `view` match arms for `Buffer::Tasks`, `Buffer::QuickFix`, and `Buffer::Help` to extract `buffer.buffer.lines.len()` and call `label_status` / `label_status_unfocused` with the appropriate label string
- [x] 2.2 Remove the 6 now-unused functions: `tasks_status`, `tasks_status_unfocused`, `quickfix_status`, `quickfix_status_unfocused`, `help_status`, `help_status_unfocused`

## 3. Verify

- [x] 3.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
