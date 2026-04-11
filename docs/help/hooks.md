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

Called for each bufferline during buffer content updates. This hook fires for **all buffer types**: directory, content (file preview), help, quickfix, and tasks. Plugins use this hook to set icons and text colors on buffer entries. The hook fires at the point where buffer content is set, so the plugin processes entries as they arrive.

When `PathsAdded` events are deferred during Insert mode, hook invocation is also deferred. Hooks fire when deferred events are flushed after leaving Insert mode.

Register callbacks with `:add()`:

```lua
y.hook.on_bufferline_mutate:add(function(ctx)
  -- Only process directory buffers
  if ctx.buffer.type ~= "directory" then
    return
  end

  local content = ctx.content or ""
  local is_directory = content:sub(-1) == "/"

  if is_directory then
    ctx.icon = "\27[94m\27[0m"
  else
    ctx.icon = "\27[37m\27[0m"
  end

  -- Prepend ANSI color to content
  ctx.content = "\27[37m" .. content .. "\27[0m"
end)
```

Each callback receives a context table with mutable bufferline fields and a read-only `buffer` metadata object:

| Field | Type | Mutable | Description |
| --- | --- | --- | --- |
| `buffer` | table | no | Read-only metadata object with `type` and `path` fields (see below) |
| `prefix` | string or nil | yes | Line prefix text |
| `content` | string | yes | Full line content as a string (may contain ANSI escape sequences) |
| `icon` | string or nil | yes | Icon glyph to render in the icon column (may include ANSI sequences for color) |

The `buffer` metadata object contains:

| Field | Type | Description |
| --- | --- | --- |
| `buffer.type` | string | Buffer type: `"directory"`, `"content"`, `"help"`, `"quickfix"`, or `"tasks"` |
| `buffer.path` | string or nil | Associated path: parent directory for directory buffers, file path for content buffers. Absent (nil) for help, quickfix, and tasks buffers. |

The `buffer` object is read-only — changes to `buffer.type` or `buffer.path` are not read back by the core. The `buffer.path` field is only present for buffer types that have an associated path (directory and content); it is nil for help, quickfix, and tasks buffers. New metadata fields may be added to `buffer` in future versions without breaking existing plugins.

After all callbacks run, `icon`, `prefix`, and `content` are read back from the context table and applied to the bufferline. The `buffer` metadata object is not read back.

- **`icon`**: Set to a string to display an icon glyph in the icon column. Include ANSI escape sequences in the string to color the icon (e.g., `"\27[38;2;222;165;132m\27[0m"`). Setting to `nil` clears the icon.
- **`content`**: Prepend ANSI escape sequences to color the filename/line text. The content string is parsed as an Ansi string, so inline ANSI sequences are preserved.
- **`prefix`**: Set to override the line prefix.
- **`buffer`**: Read-only metadata object. Modifications are ignored by the core.

If no callback modifies `icon`, the icon column renders as empty space. If no callback modifies `content`, the text uses default (unstyled) rendering.

### Buffer Type Filtering

The hook fires for every buffer type. Plugins should check `ctx.buffer.type` and return early for buffer types they do not process:

```lua
y.hook.on_bufferline_mutate:add(function(ctx)
  if ctx.buffer.type ~= "directory" then
    return
  end
  -- Process directory entries...
end)
```

### Trailing Slash Convention

In directory buffers, directory entries end with a trailing slash (`/`). Plugins detect directories by checking if `ctx.content` ends with `/`. Strip the trailing slash before performing filename-based icon resolution:

```lua
local content = ctx.content or ""
local is_directory = content:sub(-1) == "/"
local filename = is_directory and content:sub(1, -2) or content
```

### Cursor Behavior

The icon column is non-editable prefix content. The cursor starts at the first filename character, not inside the icon column. Normal and Insert mode operations apply to filename text only.
