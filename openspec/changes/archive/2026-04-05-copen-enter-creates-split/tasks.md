## 1. Create split on Enter when no directory sibling exists

- [x] 1.1 In `open::selected` in `yeet-frontend/src/update/open.rs`, when `find_nearest_directory_in_sibling` returns `None` and a path was resolved, update `current_index`, refresh the copen buffer, create a horizontal split with a new directory window above and copen below, focus the directory, and emit `NavigateToPathAsPreview`

## 2. Tests

- [x] 2.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
