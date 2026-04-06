<div align="center">
  <img src="assets/logo.svg" alt="yeet logo" width="25%">
</div>

## yeet - the vision

Yet Another Astoundingly Hackable, Keyboard-Controlled, Efficient, Versatile,
Interactive, Fast, Elmish, Minimalistic, and Superlative File Explorer with
Vim-Inspired Keybindings, Infused with the Magic of Lua, Allowing Users to Extend
Its Functionality, Shape Its Behavior, and Create Customized Workflows Tailored
to Their Unique Needs!

In short: y337

<div align="center">
  <img src="https://github.com/user-attachments/assets/4a5268ba-e796-45dc-9ae8-8a41386c0a49" alt="yeet showing an image and listing qfix entries with :cl" width="95%">
</div>

## quick start

Yeet uses four vim-inspired modes: **Navigation** (default, browse files),
**Normal** (rename/edit), **Insert** (type text), and **Command** (`:` commands).
Press `Esc` to move up the mode hierarchy. See `:help modes` for details.

| key          | action                                       |
| ------------ | -------------------------------------------- |
| `h`, `l`     | navigate parent/child directories            |
| `j`, `k`     | move cursor up/down                          |
| `Enter`      | open file or enter directory                 |
| `gg`, `G`    | jump to top/bottom                           |
| `yy`         | yank file to junk yard                       |
| `p`          | paste from junk yard                         |
| `dd`         | trash file (recoverable via junk yard)       |
| `Space`      | toggle quickfix selection                    |
| `/`, `?`     | search forward/backward                      |
| `:`          | enter command mode                           |

For the full keybinding reference, run `:help keybindings` inside yeet.

## commands

Yeet provides commands for file operations (`:w`, `:cp`, `:mv`, `:d!`), window
management (`:split`, `:vsplit`, `:q`), tabs (`:tabnew`, `:tabc`, `:tabn`),
search (`:fd`, `:rg`), quickfix (`:copen`, `:cn`, `:cN`), tasks (`:topen`),
and more.

For the full command reference, run `:help commands` inside yeet.

## cli

```sh
$ yeet --help
yeet - yet another... read the name on gh...

Usage: yeet [OPTIONS] [path]

Arguments:
  [path]  path to open in yeet on startup

Options:
      --selection-to-file-on-open <selection-to-file-on-open>
          on open write selected paths to the given file path instead and close the application
      --selection-to-stdout-on-open
          on open print selected paths to stdout instead and close the application
  -v, --verbosity <verbosity>
          set verbosity level for file logging [default: warn] [possible values: error, warn, info, debug, trace]
  -h, --help
          Print help
```

## configuration

Yeet loads `init.lua` from `$XDG_CONFIG_HOME/yeet/init.lua` (or
`~/.config/yeet/init.lua`) on startup. Use the `y.theme` table to override
theme colors:

```lua
y = {
  theme = {
    TabBarActiveBg = "#87CEFA",
    StatusLineFocusedFg = "#FFFFFF",
  }
}
```

For the full list of theme tokens, run `:help configuration` inside yeet.

## faq

### how fast is yeet

It utilizes the same mechanics like yazi (tokio i/o) without that many roundtrips
because of the underlying architecture. Thus, it should be equally fast. E.g. reading
a directory with 500k entries takes only a couple of seconds without blocking the
ui.

### image preview stays empty

All major emulator image protocols are useable. Kitty, Sixel, iTerm2 and
halfblocks are integrated with the awesome [ratatui_image](https://docs.rs/crate/ratatui-image/latest)!

If nothing of the above protocols are working, `chafa` is used as a fallback to
convert images to ansi. If the output stays empty, make sure yeet can call
`chafa` to enable image rendering.

### opening files in linux does nothing

yeet utilizes `xdg-open` to start files. Thus, not opening anything probably lies
in a misconfigured mime setup. Check `~/.local/share/applications/` for invalid entries.
Some programs causing problems regularly. Im looking at you `wine`...

## architecture overview

### yeet crate

The main crate is handling frontend and backend and resolves cli arguments to
pass them to the relevant components.

### yeet-frontend crate

The frontend follows an elm architecture with one exception: The model is
mutable and will not get created every update.

It holds the lifecycle of the tui. It starts an event stream to
enable non lockable operations. This stream is implemented in event.rs and
translates multiple event emitter like terminal interaction with crossterm into
messages.

The modules model, update and view represent parts of the elm philosophy. Messages
are defined in yeet-keymap to prevent cycling dependencies.

### yeet-buffer crate

Buffer holds all buffer relevant functionality to render content in yeet. Except
e.g. Statusline, everything is a buffer!

The create follows the elm architecture as well.

### yeet-keymap crate

This crate holds all key relevant features. The MessageResolver uses buffer
and tree to resolve possible messages, which follow the elm architecture to
modify the model.

tree uses the keymap to build a key tree structure. Thus, in keymap all
key combinations are mapped indirectly to messages.

conversion translates crossterm key events to the yeet-keymap
representation.
