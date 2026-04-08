## Why

To test the plugin system end-to-end, a real theme plugin is needed. The plugin also establishes the `setup()` pattern for plugin authoring — plugins return a module table from `init.lua` that users call `setup()` on. This requires the plugin loading system to support Lua's `require()` by adding plugin directories to `package.path` and storing returned module tables in `package.loaded`.

## What Changes

- Add `git@github.com:aserowy/yeet-bluloco-theme.git` as a git submodule at `./plugins/bluloco-theme`
- Create `init.lua` in the plugin repo with the bluloco dark theme palette and a `setup()` function
- Update plugin loading in `yeet-lua` to add each plugin's directory to `package.path` and store module return values in `package.loaded` (changing `exec()` to `eval()`)
- Add documentation for the `require()` / `setup()` plugin pattern

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `plugins`: Plugin loading now supports `require()` by adding plugin directories to `package.path` and storing return values in `package.loaded`

## Impact

- **plugins/bluloco-theme**: New git submodule with `init.lua`
- **yeet-lua/src/loading.rs**: Plugin loading adds to `package.path`, uses `eval()` instead of `exec()`, stores returned tables in `package.loaded`
- **docs/help/plugins.md**: Updated with `require()` / `setup()` pattern
- **docs/plugins.md**: Updated with authoring guide
