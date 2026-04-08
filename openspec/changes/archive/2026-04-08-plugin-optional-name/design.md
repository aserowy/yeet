## Context

`derive_plugin_name` strips `yeet-` prefix: `yeet-bluloco-theme` becomes `bluloco-theme`. This is a hidden convention. Users should control the `require()` name explicitly if they want it different from the repo name.

## Goals / Non-Goals

**Goals:**

- `name` field in register opts → stored in `PluginSpec`
- Default name = last URL segment (no prefix stripping)
- Explicit `name` overrides the default

**Non-Goals:**

- Name validation or uniqueness enforcement

## Decisions

### 1. PluginSpec gains name field

```rust
pub struct PluginSpec {
    pub url: String,
    pub name: Option<String>,
    pub branch: Option<String>,
    pub version: Option<String>,
    pub dependencies: Vec<PluginSpec>,
}
```

### 2. derive_plugin_name uses full segment

```rust
pub fn derive_plugin_name(url: &str) -> String {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    url.rsplit('/').next().unwrap_or(url).to_string()
}
```

### 3. Plugin name resolution order

In `load_plugins`: `spec.name.unwrap_or_else(|| derive_plugin_name(&spec.url))`

### 4. Lua register usage

```lua
y.plugin.register({
    url = "https://github.com/aserowy/yeet-bluloco-theme",
    name = "bluloco-theme",  -- optional override
})
require('bluloco-theme').setup()
```
