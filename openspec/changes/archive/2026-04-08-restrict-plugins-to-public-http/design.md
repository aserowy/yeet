## Context

gix by default inherits the system's git credential helpers. When a URL requires authentication, gix invokes the credential helper which may prompt on stdin — fatal for a TUI app that owns the terminal.

Plugin URLs come from `y.plugin.register({ url = "..." })` in the user's `init.lua`.

## Goals / Non-Goals

**Goals:**

- Reject non-HTTPS URLs at registration time with a clear error
- Disable gix credential prompting so auth failures produce clean errors
- Git errors during sync/update don't break yeet's state

**Non-Goals:**

- Supporting SSH or authenticated HTTPS repos for plugins

## Decisions

### 1. URL validation in register()

In the Lua `register()` function, validate that `url` starts with `https://`. Reject with `tracing::error` and skip registration for:
- `git@...` (SSH)
- `ssh://...`
- `git://...`
- `http://` (insecure — only HTTPS)
- Any other scheme

### 2. Disable credential prompting in gix

Configure gix clone operations with `configure_connection` to suppress credential helpers. Use `gix::open_opts` with `GIT_TERMINAL_PROMPT=0` equivalent, or configure the clone's `PrepareFetch` to skip credential helpers.

The simplest approach: use `gix::open::Options::isolated()` for operations that don't need the system git config, and set the `GIT_TERMINAL_PROMPT` env var to `"0"` before git operations.

### 3. Per-plugin error handling already works

The sync/update task handlers already collect per-plugin errors and report them. The issue is that gix blocks on credential prompt instead of erroring. Fixing the prompt (decision 2) makes the existing error handling work correctly.
