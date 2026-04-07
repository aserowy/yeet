## 1. Root Cargo.toml

- [x] 1.1 Add workspace-internal crates (`yeet-buffer`, `yeet-keymap`, `yeet-frontend`, `yeet-lua`) to `[workspace.dependencies]` with `path` and `version.workspace = true`

## 2. Member Cargo.toml files

- [x] 2.1 Update `yeet/Cargo.toml`: change `yeet-frontend` and `yeet-lua` dependencies to `workspace = true`
- [x] 2.2 Update `yeet-frontend/Cargo.toml`: change `yeet-buffer`, `yeet-keymap`, and `yeet-lua` dependencies to `workspace = true`
- [x] 2.3 Update `yeet-keymap/Cargo.toml`: change `yeet-buffer` dependency to `workspace = true`
- [x] 2.4 Update `yeet-lua/Cargo.toml`: change `yeet-buffer` dependency to `workspace = true`

## 3. Lock file

- [x] 3.1 Run `cargo update --workspace` to refresh `Cargo.lock`
