## Why

The codebase contains one-line comments that merely explain what the next few lines of code do. These comments add noise and become stale as code evolves. Code is the truth — if the intent isn't clear from the code itself, the code should be refactored.

## What Changes

- Remove explanatory comments from production code (non-test source files) that describe what the following lines do
- Remove commented-out code blocks
- Keep TODO/NOTE/FIX/HACK/FIXME/SAFETY markers — these serve a different purpose
- Keep section-header comments in theme.rs enum variants — these organize a large struct
- Remove explanatory comments from test code as well, keeping only comments that document non-obvious test setup rationale

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `devops`: Add a coding standard that prohibits explanatory comments in source code

## Impact

- All Rust source files across yeet, yeet-buffer, yeet-frontend, yeet-keymap, yeet-lua, yeet-plugin
