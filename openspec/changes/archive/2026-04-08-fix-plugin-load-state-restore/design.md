## Context

`take_snapshot` records `hook_count: usize` and `theme_keys: Vec<String>`. `restore_snapshot` nils out hook indices beyond `hook_count` and removes new theme keys. This works only because `:add()` always appends. But it's fragile and the code is harder to reason about than a simple clone-and-swap.

`LuaTable` in mlua is a reference type — Rust `.clone()` clones the reference, not the data. A shallow copy requires iterating entries into a new table.

## Goals / Non-Goals

**Goals:**

- Snapshot clones hook and theme table contents into fresh Lua tables
- Restore replaces live table contents from the clone (clear + copy back)
- Existing tests pass; new test proves earlier plugin hooks survive later plugin failure

**Non-Goals:**

- Deep-cloning nested tables within hook callbacks (functions are reference types in Lua, shallow copy is sufficient)

## Decisions

### 1. Clone via shallow copy into new Lua table

`take_snapshot` creates a new Lua table and copies all key-value pairs from `y.hook.on_window_create` and `y.theme` into it. Functions (hook callbacks) are Lua reference types so a shallow copy preserves them correctly.

### 2. Restore via clear + copy back

`restore_snapshot` clears all entries in the live table (iterate keys, set nil), then copies all entries from the snapshot table back. This is correct regardless of what the failed plugin did (append, replace, delete).

### 3. PluginSnapshot holds LuaTable clones

```rust
struct PluginSnapshot {
    hook_on_window_create: LuaTable,
    theme: LuaTable,
}
```

No more `hook_count` or `theme_keys` — just the cloned tables.
