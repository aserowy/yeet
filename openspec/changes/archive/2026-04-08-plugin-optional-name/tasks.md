## 1. PluginSpec name field

- [x] 1.1 Add `name: Option<String>` to `PluginSpec` in `yeet-plugin/src/spec.rs`

## 2. Lua registration

- [x] 2.1 Read `name` field in `register()` function in `yeet-lua/src/plugin.rs` and store in plugin entry
- [x] 2.2 Read `name` field in `read_plugin_specs()` and populate `PluginSpec.name`

## 3. Plugin name derivation

- [x] 3.1 Remove `yeet-` prefix stripping from `derive_plugin_name` in `loading.rs` — use last URL segment as-is
- [x] 3.2 Use `spec.name.unwrap_or_else(|| derive_plugin_name(&spec.url))` when resolving the plugin name in `load_plugins`
- [x] 3.3 Update tests for `derive_plugin_name` (no prefix stripping) and add test for explicit name override

## 4. Documentation

- [x] 4.1 Update `docs/help/plugins.md` to document `name` option and that default uses full repo name

## 5. Build Verification

- [x] 5.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
