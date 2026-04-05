## Why

Yeet has no built-in help system. Users must leave the application to consult the README or external documentation. A `:help` command — similar to Neovim's help pages — would let users discover keybindings, commands, and features without leaving the editor, making the application more self-documenting and approachable.

## What Changes

- Add a `:help` command that opens a help index page as a read-only markdown buffer in a horizontal split below the current window
- Add `:help <topic>` to jump directly to a specific help page or section
- Bundle help pages as markdown files shipped with the application
- Introduce a new `Help` buffer type for rendering markdown content read-only
- Add a new `Window::Help` variant (or reuse existing split mechanics) to display help in a dedicated pane

## Capabilities

### New Capabilities

- `help-command-dispatch`: The `:help` and `:help <topic>` command parsing, topic lookup/resolution, and split creation
- `help-buffer`: Help buffer behavior — read-only mode, navigation keybindings, and standard close with `:q`
- `help-content`: Bundled markdown help pages embedded at compile time

### Modified Capabilities

_(none — splits and window management are reused as-is, no spec-level behavior changes)_

## Impact

- **Command dispatch** (`yeet-frontend/src/update/command/mod.rs`): New `:help` command entry
- **Buffer model** (`yeet-frontend/src/model/mod.rs`): New `Buffer::Help` variant or reuse of `ContentBuffer` in read-only mode
- **Window model** (`yeet-frontend/src/model/mod.rs`): Potentially new `Window` variant or reuse of existing split infrastructure
- **View rendering** (`yeet-frontend/src/view/`): Rendering logic for help buffer content
- **Build/packaging**: Help markdown files need to be embedded or shipped alongside the binary
