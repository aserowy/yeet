## 1. Core Implementation

- [x] 1.1 Add a `close_focused` method on `Window` that recursively follows the focus path, replaces the innermost split containing the focused leaf with its sibling, and returns the dropped subtree
- [x] 1.2 Refactor `close_focused_window_or_quit` in `yeet-frontend/src/update/command/mod.rs` to call `Window::close_focused` instead of pattern matching at the root level

## 2. Testing

- [x] 2.1 Add tests for closing a focused leaf in a single-level horizontal split
- [x] 2.2 Add tests for closing a focused leaf in a single-level vertical split
- [x] 2.3 Add tests for closing a focused leaf in a doubly-nested split (split within split)
- [x] 2.4 Add tests for closing when the root window is a leaf (should return None/indicate quit)
- [x] 2.5 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify all checks pass
