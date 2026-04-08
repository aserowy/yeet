## Context

Lua's `require()` uses `package.searchers` (an ordered list of functions) to find modules. If no searcher finds the module, `require()` errors. We can prepend a custom searcher that, for registered plugin names, returns a no-op proxy table instead of failing.

The proxy needs a `__index` metamethod that returns a no-op function for any key access, so `require('x').setup()` or `require('x').anything()` silently does nothing.

## Goals / Non-Goals

**Goals:**

- `require('plugin-name')` returns a no-op proxy when the plugin is registered but not loaded
- `require('plugin-name').setup()` works without error on fresh install
- Once loaded, `require()` returns the real module (already in `package.loaded`)

**Non-Goals:**

- Making the proxy do anything useful (it's a silent no-op until the real plugin loads)

## Decisions

### 1. Custom package searcher in Lua

After `setup_and_execute` creates the `y.plugin` table, register a custom searcher at `package.searchers[2]` (before the file searcher) that:

1. Gets the module name
2. Checks if any registered plugin has that name (or derives to that name)
3. If yes and `package.loaded[name]` is nil: return a loader function that creates and returns a no-op proxy
4. If no: return nil (let other searchers handle it)

Actually simpler: since `register()` is called during `init.lua` and `require()` may be called later in the same script, the searcher needs access to the live `_plugins` table. The easiest implementation: add the searcher in Lua code within `setup_and_execute`, not in Rust.

### 2. No-op proxy via __index metamethod

```lua
local proxy_mt = {
    __index = function(_, _)
        return function() end
    end
}
```

Any method call on the proxy returns a function that does nothing. Nested calls like `require('x').y.z()` won't work (only one level deep), but `require('x').setup()` does — which is the expected pattern.

### 3. Searcher checks registered names

The searcher iterates `y.plugin._plugins`, derives the name for each (last URL segment, or explicit `name` field), and matches against the requested module name.
