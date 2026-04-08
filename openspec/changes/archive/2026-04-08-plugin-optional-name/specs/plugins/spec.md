## MODIFIED Requirements

### Requirement: Plugin registration via y.plugin.register()

The `opts` table SHALL accept an additional optional field:

- `name` (string, optional): Override the plugin's `require()` name. Defaults to the last URL path segment (with `.git` suffix stripped).

#### Scenario: Register with explicit name

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-theme", name = "theme" })`
- **THEN** the plugin SHALL be accessible via `require('theme')`

#### Scenario: Register without name uses URL segment

- **WHEN** `init.lua` contains `y.plugin.register({ url = "https://github.com/user/yeet-theme" })`
- **THEN** the plugin SHALL be accessible via `require('yeet-theme')`

### Requirement: Plugin name derived from URL

- **WHEN** a plugin is registered with URL `https://github.com/aserowy/yeet-bluloco-theme`
- **THEN** the derived plugin name for `require()` SHALL be `yeet-bluloco-theme` (last URL segment, no prefix stripping)
