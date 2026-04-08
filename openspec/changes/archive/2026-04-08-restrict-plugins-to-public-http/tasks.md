## 1. URL validation in register()

- [x] 1.1 In the Lua `register()` function in `plugin.rs`, validate that `url` starts with `https://` before accepting registration
- [x] 1.2 Log error and skip registration for `git@`, `ssh://`, `git://`, `http://`, and any other scheme

## 2. Disable credential prompting in gix

- [x] 2.1 Configure gix clone operations with `with_in_memory_config_overrides(["credential.helper="])` to disable credential helpers
- [x] 2.2 Verify that auth failures produce a `GitError` instead of blocking

## 3. Tests

- [x] 3.1 Add Lua test: `register()` with SSH URL is rejected (plugin list stays empty)
- [x] 3.2 Add Lua test: `register()` with HTTP URL is rejected
- [x] 3.3 Add Lua test: `register()` with HTTPS URL succeeds

## 4. Build Verification

- [x] 4.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
