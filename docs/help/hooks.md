# Hooks

## `y.hook`

The `y.hook` table is the namespace for lifecycle hooks. Each hook supports multiple callbacks registered via `:add()`. Callbacks are invoked in registration order when the hook fires. If no callbacks are registered, yeet uses its built-in defaults.

Errors in individual callbacks are caught gracefully — yeet logs the error and continues invoking the remaining callbacks.

## `y.hook.on_window_create`

Called whenever a new window is created. Each registered callback receives a context table describing the window and its viewport settings. Modify the viewport fields in the context table to override defaults. Mutations from earlier callbacks are visible to later ones.

Register callbacks with `:add()`:

```lua
y.hook.on_window_create:add(function(ctx)
  if ctx.type == "directory" then
    ctx.current.line_number = "absolute"
    ctx.current.wrap = true
    ctx.preview.hide_cursor_line = false
    ctx.parent.show_border = false
  end
end)
```

Multiple callbacks can be registered for the same hook:

```lua
y.hook.on_window_create:add(function(ctx)
  if ctx.type == "help" then
    ctx.viewport.wrap = true
  end
end)

y.hook.on_window_create:add(function(ctx)
  if ctx.type == "quickfix" then
    ctx.viewport.line_number = "absolute"
    ctx.viewport.line_number_width = 3
  end
end)
```

The `ctx.type` field is one of: `"directory"`, `"help"`, `"quickfix"`, `"tasks"`.

The `ctx.path` field contains the target path for directory windows (if known at creation time), or nil.

For directory windows, the context has `parent`, `current`, and `preview` subtables. For help, quickfix, and tasks windows, the context has a single `viewport` subtable.

Each viewport settings subtable contains:

| Field | Type | Values | Default (current pane) |
| --- | --- | --- | --- |
| `line_number` | string | `"none"`, `"absolute"`, `"relative"` | `"relative"` |
| `line_number_width` | integer | >= 0 | 3 |
| `sign_column_width` | integer | >= 0 | 2 |
| `show_border` | boolean | | true |
| `hide_cursor` | boolean | | false |
| `hide_cursor_line` | boolean | | false |
| `wrap` | boolean | | false |

Invalid values (wrong type or unrecognized strings) are ignored and the default is kept. Unknown fields are silently ignored.
