## 1. Git Submodule

- [x] 1.1 Add `git@github.com:aserowy/yeet-bluloco-theme.git` as submodule at `./plugins/bluloco-theme`

## 2. Plugin init.lua

- [x] 2.1 Create `plugins/bluloco-theme/init.lua` with bluloco dark palette and `setup()` function that applies `y.theme` assignments
- [x] 2.2 Call `setup()` automatically at the end of `init.lua` so the theme applies on load

## 3. Plugin loading: require() support

- [x] 3.1 Add `derive_plugin_name(url)` function that extracts the last URL segment and strips `yeet-` prefix
- [x] 3.2 Before executing a plugin's `init.lua`, prepend its directory to `package.path`
- [x] 3.3 Change `exec()` to `eval::<LuaValue>()` in `load_single_plugin`
- [x] 3.4 If the eval result is a non-nil table, store it in `package.loaded[plugin_name]`
- [x] 3.5 Add tests for `derive_plugin_name` and `require()` integration

## 4. Documentation

- [x] 4.1 Update `docs/help/plugins.md` with the `require()` / `setup()` pattern and example
- [x] 4.2 Update `docs/plugins.md` with plugin authoring guide showing the module return pattern (N/A — no standalone file, help page is canonical)

## 5. Build Verification

- [x] 5.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 5.2 Run `cargo test` and ensure all tests pass
