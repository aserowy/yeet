## 1. Pass data path to Lua

- [x] 1.1 In `setup_and_execute`, resolve the plugin data path and set `y.plugin._data_path` before executing `init.lua`

## 2. Update custom searcher

- [x] 2.1 Add `url_to_storage` Lua function in the searcher that replicates the Rust `url_to_storage_path` logic
- [x] 2.2 Update the searcher: when a registered plugin matches, check if `_data_path/<storage>/init.lua` exists on disk
- [x] 2.3 If exists: `dofile()` the init.lua, store result in `package.loaded[modname]`, return the real module
- [x] 2.4 If not exists: return no-op proxy (existing behavior)

## 3. Skip already-loaded plugins in load_plugins

- [x] 3.1 In `load_plugins`, check `package.loaded[plugin_name]` before executing a plugin's `init.lua` — skip if already loaded

## 4. Tests

- [x] 4.1 Add test: plugin on disk + `require().setup()` in init.lua → theme values persist
- [x] 4.2 Add test: plugin not on disk + `require().setup()` in init.lua → no-op proxy, no error
- [x] 4.3 Verify plugin loaded via require() is not double-loaded by load_plugins

## 5. Build Verification

- [x] 5.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
