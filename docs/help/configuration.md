# Configuration

Yeet loads a Lua configuration file on startup to customize theme colors, syntax highlighting, and register lifecycle hooks. If no configuration file exists, yeet starts with built-in defaults.

## Config File

### `Location`

The configuration file is `init.lua` and is loaded from the XDG config directory. Yeet checks `$XDG_CONFIG_HOME/yeet/init.lua` first, then falls back to `~/.config/yeet/init.lua` if the environment variable is not set.

### `Error handling`

If `init.lua` contains syntax errors or runtime errors, yeet logs the error and continues startup with default settings. The application will not crash due to a broken configuration file.

## Topics

- `:help theme` — theme colors, syntax highlighting, and all color token references
- `:help hooks` — lifecycle hooks (`y.hook`) for customizing yeet behavior
