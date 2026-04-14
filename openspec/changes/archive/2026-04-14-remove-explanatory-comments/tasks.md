## 1. Remove explanatory comments from production code

- [x] 1.1 Remove explanatory comments from `yeet-lua/src/hook.rs`
- [x] 1.2 Remove commented-out code and explanatory comments from `yeet-keymap/src/conversion.rs`
- [x] 1.3 Remove explanatory comments from `yeet-frontend/src/update/path.rs`
- [x] 1.4 Remove explanatory comments from `yeet-frontend/src/update/command/help.rs`
- [x] 1.5 Remove explanatory comments from `yeet-frontend/src/update/command/file.rs`
- [x] 1.6 Remove commented-out code from `yeet-frontend/src/action.rs`
- [x] 1.7 Remove explanatory comments from `yeet-frontend/src/update/window.rs` (production code only)

## 2. Remove explanatory comments from test code

- [x] 2.1 Remove explanatory comments from test code in `yeet-buffer/src/view/mod.rs`
- [x] 2.2 Remove explanatory comments from test code in `yeet-buffer/src/model/viewport.rs`
- [x] 2.3 Remove explanatory comments from test code in `yeet-buffer/src/model/undo.rs`
- [x] 2.4 Remove explanatory comments from test code in `yeet-frontend/src/theme.rs`
- [x] 2.5 Remove explanatory comments from test code in `yeet-frontend/src/view/buffer.rs`
- [x] 2.6 Remove explanatory comments from test code in `yeet-frontend/src/update/focus.rs`
- [x] 2.7 Remove explanatory comments from test code in `yeet-frontend/src/update/command/mod.rs`
- [x] 2.8 Remove explanatory comments from test code in `yeet-frontend/src/update/command/task.rs`
- [x] 2.9 Remove explanatory comments from test code in `yeet-frontend/src/update/modify.rs`
- [x] 2.10 Remove explanatory comments from test code in `yeet-frontend/src/update/task.rs`
- [x] 2.11 Remove explanatory comments from test code in `yeet-frontend/src/update/mode.rs`
- [x] 2.12 Remove explanatory comments from test code in `yeet-frontend/src/update/navigate.rs`
- [x] 2.13 Remove explanatory comments from test code in `yeet/src/lua.rs`

## 3. Validation

- [x] 3.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 3.2 Run `cargo test` and fix any failures
- [x] 3.3 Run `git add -A && nix build .` and fix any build errors
