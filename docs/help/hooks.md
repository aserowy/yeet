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
| `icon_column_width` | integer | >= 0 | 0 |
| `line_number` | string | `"none"`, `"absolute"`, `"relative"` | `"relative"` |
| `line_number_width` | integer | >= 0 | 3 |
| `sign_column_width` | integer | >= 0 | 2 |
| `show_border` | boolean | | true |
| `hide_cursor` | boolean | | false |
| `hide_cursor_line` | boolean | | false |
| `wrap` | boolean | | false |

Invalid values (wrong type or unrecognized strings) are ignored and the default is kept. Unknown fields are silently ignored.

### Icon Column

The `icon_column_width` field controls how many cells are reserved for the icon column in the buffer prefix. It defaults to `0` (no icon column). The `yeet-directory-icons` plugin sets this to `1` via `on_window_create` to enable icon rendering:

```lua
y.hook.on_window_create:add(function(ctx)
  if ctx.type == "directory" then
    ctx.parent.icon_column_width = 1
    ctx.current.icon_column_width = 1
    ctx.preview.icon_column_width = 1
  end
end)
```

## `y.hook.on_bufferline_mutate`

Called for each bufferline during directory content updates (`EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling). Plugins use this hook to set icons and text colors on directory entries. The hook fires at the same point where directory content is set, so the plugin processes entries as they arrive.

When `PathsAdded` events are deferred during Insert mode, hook invocation is also deferred. Hooks fire when deferred events are flushed after leaving Insert mode.

Register callbacks with `:add()`:

```lua
y.hook.on_bufferline_mutate:add(function(ctx)
  if ctx.is_directory then
    ctx.icon = ""
    ctx.icon_style = "\27[94m"
  else
    ctx.icon = ""
    ctx.icon_style = "\27[37m"
  end
end)
```

Each callback receives a context table with:

| Field | Type | Mutable | Description |
| --- | --- | --- | --- |
| `filename` | string | no | Display name of the directory entry |
| `is_directory` | boolean | no | Whether the entry is a directory |
| `icon` | string or nil | yes | Icon glyph to render in the icon column |
| `icon_style` | string or nil | yes | ANSI foreground escape sequence for icon and filename text |

After all callbacks run, `icon` and `icon_style` are read back from the context table and applied to the bufferline. Setting `icon_style` applies the color to both the icon glyph and the filename text. Setting either field to `nil` clears it.

If no callback modifies `icon`, the icon column renders as empty space. If no callback modifies `icon_style`, the filename text uses default styling.

### Cursor Behavior

The icon column is non-editable prefix content. The cursor starts at the first filename character, not inside the icon column. Normal and Insert mode operations apply to filename text only.
