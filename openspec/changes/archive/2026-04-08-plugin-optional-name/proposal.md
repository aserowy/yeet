## Why

`derive_plugin_name` currently strips the `yeet-` prefix from the last URL segment to produce the `require()` name. This is a magic convention that isn't obvious to plugin authors. Instead, the plugin name for `require()` should default to the full last URL segment (e.g., `yeet-bluloco-theme`), and users should be able to override it with an explicit `name` field in `register()`.

## What Changes

- Add `name` (string, optional) to the `register()` opts and `PluginSpec` struct
- Remove `yeet-` prefix stripping from `derive_plugin_name` — use last URL segment as-is
- When loading a plugin, use `spec.name` if set, otherwise fall back to `derive_plugin_name(url)`
- Update docs and tests

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `plugins`: `register()` accepts optional `name` field; plugin name derivation no longer strips `yeet-` prefix

## Impact

- **yeet-plugin/src/spec.rs**: `PluginSpec` gains `name: Option<String>`
- **yeet-lua/src/plugin.rs**: `register()` reads `name` field; `read_plugin_specs` returns it
- **yeet-lua/src/loading.rs**: `derive_plugin_name` simplified; uses `spec.name` when available
