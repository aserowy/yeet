## 1. Add focus-shifting helper

- [x] 1.1 Add `focus_nearest_directory` function in `yeet-frontend/src/update/command/qfix/window.rs` that flips `SplitFocus` from the QuickFix child to the sibling directory child, recursively finding the split that directly contains the QuickFix window

## 2. Fix Enter handler

- [x] 2.1 In `open::selected` in `yeet-frontend/src/update/open.rs`, call `focus_nearest_directory` on the window tree before emitting `NavigateToPathAsPreview`, so focus moves to the directory window and the navigation can find the directory viewports

## 3. Tests

- [x] 3.1 Add a test that verifies `focus_nearest_directory` flips focus from QuickFix to the sibling directory
- [x] 3.2 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
