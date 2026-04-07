## Context

The root `Cargo.toml` contains three categories of `version = "..."` entries:
1. `[workspace.package]` → `version = "2026.3.0"` (must update)
2. `[workspace.dependencies]` internal crates → `yeet-buffer = { path = "yeet-buffer", version = "2026.3.0" }` (must update)
3. `[workspace.dependencies]` external crates → `arboard = { version = "3.6.1", ... }` (must NOT update)

The current `sed` pattern matches all three.

## Goals / Non-Goals

**Goals:**

- Only replace version strings on lines that belong to the workspace package or internal crate entries
- Keep the approach simple and maintainable

**Non-Goals:**

- Changing the version format or computation logic

## Decisions

### 1. Use two targeted sed commands instead of one blanket replacement

First `sed`: replace the `version` line that starts with `version =` (the `[workspace.package]` entry — it's the only line in the file where `version` is the first token).

Second `sed`: replace `version = "..."` only on lines that start with `yeet-` (the internal crate entries).

**Alternative considered**: Using a TOML-aware tool — rejected as unnecessarily complex for a focused fix. The two patterns are unambiguous and won't match external deps.

## Risks / Trade-offs

- **If an external dep name starts with `yeet-`** → Would be incorrectly matched. This is unlikely since all `yeet-*` crates are workspace-internal.
