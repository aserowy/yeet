## Context

`setup_and_execute` creates `y` with `y.theme` (plain table) and `y.hook` (table with hook objects that have metatables for `:add()`). It then sets `y` on `lua.globals()` and executes `init.lua`. If the user writes `y = { theme = { ... } }`, Lua replaces the global `y` entirely — the Rust-created `y.hook` table is lost.

## Goals / Non-Goals

**Goals:**

- `y = { theme = { ... } }` merges into the existing `y` table, preserving `y.hook`
- `y.theme = { ... }` replaces just the theme subtable (existing behavior, unchanged)
- `y.theme.X = "..."` sets a single value (existing behavior, unchanged)
- Metatables on `y.hook` subtables survive any form of assignment

**Non-Goals:**

- Deep-merging `y.hook` subtables (e.g., `y = { hook = { ... } }` replaces `y.hook` entirely — hooks are managed by the system, not user tables)
- Protecting subtables other than `y` itself

## Decisions

### 1. Metatable on `_G` with `__newindex`

**Decision:** After creating `y` and setting it on globals, set a metatable on `_G` with a `__newindex` metamethod. When the key is `"y"` and the value is a table, shallow-merge the new table's keys into the existing `y` table. For any other key, use `rawset` to allow normal global variable creation.

**Rationale:** This is the only interception point that catches `y = { ... }` at the top level. The `__newindex` metamethod fires when setting a key that doesn't already exist — but since `y` already exists, we need to use `__newindex` on a proxy or use `rawset` to initially set `y`, then protect reassignment.

**Revised approach:** After setting `y` via `rawset`, set a metatable on `_G` where `__newindex` intercepts writes. Since `y` already exists (set via `rawset`), normal `_G.y = ...` would go through `__newindex` only if we remove `y` from `_G` and store it in the metatable's `__index`. This is overly complex.

**Simpler approach:** After `init.lua` executes, read the user's `y` table and merge anything new into the Rust-created one. But this doesn't help if `init.lua` overwrites `y` before accessing `y.hook`.

**Chosen approach:** Use a `__newindex` on `_G` that fires for `y` assignments. To make this work, store the real `y` table in a separate location (not directly on `_G`), and use `__index` on `_G`'s metatable to return it when `y` is read. `__newindex` intercepts `y = { ... }` and merges. All other globals pass through via `rawset`.

Implementation:
1. Create `y` table with `theme` and `hook`
2. Create a `_G` metatable with:
   - `__index`: if key is `"y"`, return the protected `y` table
   - `__newindex`: if key is `"y"` and value is a table, shallow-merge value into protected `y`; if key is `"y"` and value is not a table, log warning and ignore; otherwise `rawset(_G, key, value)`
3. Do NOT set `y` directly on `_G` — store it only in the metatable closure

This ensures `y` is always the Rust-created table. `y = { theme = { ... } }` merges `theme` into it. `y.hook` survives.

## Risks / Trade-offs

**Performance** → The metatable on `_G` adds a lookup indirection for every global access. For a config file that runs once at startup, this is negligible.

**Surprising merge semantics** → `y = { theme = { ... } }` doesn't replace `y` but merges. Users expecting a full reset won't get one. Mitigation: document the behavior.

**Non-table assignment to y** → `y = nil` or `y = "string"` would be silently ignored. Mitigation: log a warning.
