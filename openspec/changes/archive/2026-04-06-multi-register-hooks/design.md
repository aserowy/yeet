## Context

Hooks are currently single functions assigned to `y.hook.<name>`. The invocation code in `hook.rs` reads `y.hook.on_window_create` as a `LuaValue`, checks if it's a function, and calls it. The `y.hook` table is a plain Lua table created during init in `setup_and_execute`.

## Goals / Non-Goals

**Goals:**

- Allow multiple callbacks per hook via `y.hook.<name>:add(fn)`
- Invoke all registered callbacks in registration order
- Mutations from earlier callbacks are visible to later ones (shared context table)
- Gracefully handle errors in individual callbacks without aborting the remaining ones

**Non-Goals:**

- Removing or reordering callbacks after registration
- Priority or weight-based ordering
- Async callback execution

## Decisions

### 1. Hook objects as Lua tables with metatable

**Decision:** Each hook name (`y.hook.on_window_create`) is a Lua table with a metatable that provides an `:add()` method. Registered callbacks are stored in the array part of the table.

**User-facing API:**

```lua
y.hook.on_window_create:add(function(ctx)
  if ctx.type == "directory" then
    ctx.current.line_number = "absolute"
  end
end)

y.hook.on_window_create:add(function(ctx)
  if ctx.type == "help" then
    ctx.viewport.wrap = true
  end
end)
```

**Rationale:** Lua tables with metatables are the idiomatic way to create object-like APIs. The `:add()` method call syntax is natural Lua. Storing callbacks in the array part preserves insertion order and allows simple iteration with `ipairs`.

**Alternatives considered:**
- *Global `y.hook.add("on_window_create", fn)` function*: Less idiomatic, doesn't read as naturally.
- *Plain tables where users append*: `table.insert(y.hook.on_window_create, fn)` — requires users to know internals. The `:add()` method is a cleaner API.

### 2. Hook table initialization

**Decision:** During Lua init, create a shared metatable with the `add` method. Create each hook name as a table with this metatable. The `add` method appends the function to the table's array.

```lua
-- Conceptual Lua equivalent of what Rust sets up:
local hook_mt = {
  __index = {
    add = function(self, fn)
      table.insert(self, fn)
    end
  }
}
y.hook.on_window_create = setmetatable({}, hook_mt)
```

This is set up in Rust via mlua during `setup_and_execute`.

### 3. Invocation: iterate and call each callback

**Decision:** When invoking a hook, iterate the table's array part with `ipairs`-style indexing (1, 2, 3, ...) and call each function with the context table. If a callback errors, log the error and continue to the next callback. The context table is shared — mutations accumulate.

**Rationale:** Continuing past errors ensures one broken callback doesn't block others. The shared context table lets callbacks build on each other's work.

### 4. Error isolation per callback

**Decision:** Each callback is called individually in a protected call. If it errors, the error is logged with the callback's index and the hook name, then invocation continues with the next callback.

### 5. Non-function values in :add() are rejected

**Decision:** If `:add()` is called with a non-function argument, log a warning and ignore the call. Do not add it to the list.

## Risks / Trade-offs

**Breaking change** → Existing `y.hook.on_window_create = fn` assignments will overwrite the hook table entirely, losing the `:add()` method. Mitigation: document the migration clearly. The hook system is new (added this session) so the user base impact is minimal.

**Callback ordering dependency** → Later callbacks see mutations from earlier ones. This is intentional but could surprise users if callbacks have conflicting settings. Mitigation: document that callbacks run in registration order with shared state.
