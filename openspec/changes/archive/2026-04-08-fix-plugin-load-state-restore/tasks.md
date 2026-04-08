## 1. Implement clone-based snapshot

- [x] 1.1 Add `shallow_clone_table(lua, table)` helper that creates a new Lua table with all key-value pairs copied from the source
- [x] 1.2 Rewrite `take_snapshot` to clone `y.hook.on_window_create` and `y.theme` into new tables using the helper
- [x] 1.3 Change `PluginSnapshot` struct to hold `hook_on_window_create: LuaTable` and `theme: LuaTable`

## 2. Implement clone-based restore

- [x] 2.1 Add `restore_table_from_clone(lua, target, source)` helper that clears all entries in `target` then copies all entries from `source`
- [x] 2.2 Rewrite `restore_snapshot` to use `restore_table_from_clone` for both hook and theme tables

## 3. Tests

- [x] 3.1 Add test: plugin A registers a hook, plugin B fails — verify plugin A's hook still exists after rollback
- [x] 3.2 Add test: plugin A sets a theme color, plugin B overrides it then fails — verify plugin A's color is restored

## 4. Build Verification

- [x] 4.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 4.2 Run `cargo test` and ensure all tests pass
