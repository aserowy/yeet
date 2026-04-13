## 1. Chafa Escape Sequence Sanitization

- [ ] 1.1 Add a `strip_non_sgr_escape_sequences` function in `yeet-frontend/src/task/image.rs` that removes all CSI escape sequences not ending in `m` from a string, preserving only SGR color/style sequences
- [ ] 1.2 Apply the sanitization function to chafa's stdout in `load_with_chafa` before splitting into lines and returning as `Preview::Content`
- [ ] 1.3 Add unit tests for the sanitization function covering: SGR-only input (preserved), cursor movement sequences (stripped), erase sequences (stripped), mixed SGR and non-SGR (only SGR preserved), plain text without escapes (unchanged)

## 2. Chafa Viewport Width Fix

- [ ] 2.1 Modify the rect construction in `Action::Load` in `yeet-frontend/src/action.rs` to compute the content-area width by subtracting viewport offsets (using a default `BufferLine` to compute `get_content_width` or manually subtracting sign_column_width + line_number_width + prefix_column_width + border) before passing to `Task::LoadPreview`
- [ ] 2.2 Add a test verifying that the content-area rect width passed to `Task::LoadPreview` is less than or equal to the raw viewport width when the preview viewport has a border enabled

## 3. Validation

- [ ] 3.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [ ] 3.2 Run `cargo test` and fix any failing tests
- [ ] 3.3 Run `git add -A && nix build .` and fix any build errors
