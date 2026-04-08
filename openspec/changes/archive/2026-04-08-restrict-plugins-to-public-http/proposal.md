## Why

When a plugin is registered with a private repo URL (SSH `git@...` or private HTTPS), gix attempts to prompt for credentials on stdin. Since yeet is a TUI app, this blocks the terminal and leaves the application in a broken state. Only public HTTP(S) repositories should be allowed for plugins.

## What Changes

- Validate plugin URLs at registration time: only `https://` URLs are accepted; reject `git@`, `ssh://`, `git://`, and other schemes with a clear error
- Configure gix to never prompt for credentials: disable credential helpers so auth failures produce clean errors instead of blocking prompts
- Ensure git operation errors during sync/update are caught cleanly and reported per-plugin without breaking yeet's state

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `plugins`: Plugin registration rejects non-HTTPS URLs; gix configured without credential prompting

## Impact

- **yeet-lua/src/plugin.rs**: URL validation in `register()` — reject non-https URLs
- **yeet-plugin/src/git.rs**: Configure gix clone/fetch to disable credential prompts
