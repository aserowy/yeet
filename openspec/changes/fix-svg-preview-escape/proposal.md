## Why

When previewing files that contain extremely long lines (e.g., `assets/logo.svg` with a 517K-character base64-encoded line), `syntect`'s `as_24_bit_terminal_escaped()` produces a massive ANSI-escaped string that, when rendered through `ansi_to_tui`, can generate malformed or pathological terminal output. This causes the terminal to exit raw mode / alternate screen, making the preview appear empty and all keyboard input to bypass yeet and go directly to the shell's stdin buffer. After killing yeet, accumulated input is executed in the console — enabling remote code execution via crafted file content.

## What Changes

- Sanitize syntax-highlighted output the same way chafa output is already sanitized — strip non-SGR escape sequences from highlighted text before storing it in the buffer pipeline
- Add a line-length limit to the syntax highlighter so that extremely long lines are truncated before highlighting, preventing pathological ANSI string generation
- Move the existing `strip_non_sgr_escape_sequences` function to a shared location so both the chafa and syntax highlight paths can use it

## Capabilities

### New Capabilities

- `syntax-highlight-sanitization`: Sanitization and safety limits for syntax-highlighted preview content to prevent terminal escape and remote code execution

### Modified Capabilities

## Impact

- `yeet-frontend/src/task/syntax.rs` — add line-length truncation and post-highlight sanitization
- `yeet-frontend/src/task/image.rs` — extract `strip_non_sgr_escape_sequences` to shared module
- `yeet-frontend/src/task/mod.rs` or new `yeet-frontend/src/task/sanitize.rs` — shared sanitization utilities
