## 1. Update invoke_on_window_change signature in yeet-lua

- [x] 1.1 In `yeet-lua/src/hook.rs`, change `invoke_on_window_change` and `try_invoke_on_window_change` to accept three optional paths (`parent_path`, `current_path`, `preview_path`) instead of a single `path: Option<&Path>`. Remove the top-level `ctx.path` and instead set `path` on each viewport subtable after `build_context` creates them
- [x] 1.2 Update `invoke_on_window_change` export in `yeet-lua/src/lib.rs` to match the new signature
- [x] 1.3 Update unit tests in `yeet-lua/src/hook.rs` to verify per-viewport paths are set on subtables and top-level `ctx.path` is nil

## 2. Update invoke_on_window_change_for_focused in yeet-frontend

- [x] 2.1 In `yeet-frontend/src/update/hook.rs`, update `invoke_on_window_change_for_focused` to resolve paths for all three viewports (parent, current, preview) from their buffer IDs and pass them to `invoke_on_window_change`
- [x] 2.2 Update integration tests in `yeet-frontend/src/update/hook.rs` for per-viewport path context

## 3. Update documentation

- [x] 3.1 Update `docs/help/hooks.md` to document per-viewport `path` property on `on_window_change` context and removal of top-level `ctx.path`

## 4. Validation

- [x] 4.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 4.2 Run `cargo test` and fix any failures
- [x] 4.3 Run `git add -A && nix build .` and fix any build errors
