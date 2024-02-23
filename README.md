<div align="center">
  <img src="assets/logo.svg" alt="yeet logo" width="20%">
</div>

## yeet - the vision

Yet Another Astoundingly Hackable, Keyboard-Controlled, Efficient, Versatile,
Interactive, Fast, Elmish, Minimalistic, and Superlative File Explorer with
Vim-Inspired Keybindings, Infused with the Magic of Lua, Allowing Users to Extend
Its Functionality, Shape Its Behavior, and Create Customized Workflows Tailored
to Their Unique Needs!

In short: y337

## shortcuts

### changing modes

In every mode `esc` switches to the next 'level' mode. The order is:

navigation < normal < insert

Exception to this order is the command mode. Leaving this mode will restore the
previous one.

When transition from normal to navigation all changes to the filesystem will get
persisted. Thus, changes in insert and normal are handled like unsaved buffer changes
and are not present on the file system till `:w` gets called or the mode changes
to navigation.

### navigation mode

| keys | action                   |
| ---- | ------------------------ |
| h, l | navigating the file tree |
| gh   | goto home directory      |
| m    | go into normal mode      |

### navigation and normal mode

| keys       | action                                                    |
| ---------- | --------------------------------------------------------- |
| j, k       | navigating the current directory down/up                  |
| o, O       | add a new line and change to insert mode                  |
| i, a       | change to insert mode                                     |
| I, A       | jump to line start/end and change to insert mode          |
| dd         | go into normal and trash\* the current line               |
| yy         | yank file to register                                     |
| p          | paste " register to current path                          |
| :          | change to command mode                                    |
| zt, zz, zb | move viewport to start, center, bottom of cursor position |
| C-u, C-d   | move viewport half screen up/down                         |

\*trash: files are not deleted but moved to yeets cache folder to enable register
interactions. To delete the selected path completly, call command `:d!`.

### normal mode

| keys | action                        |
| ---- | ----------------------------- |
| h, l | move cursor left/right        |
| 0, $ | move cursor to line start/end |

## commands

| :       | action                              |
| ------- | ----------------------------------- |
| d!      | delete selected file/directory      |
| e!      | reload current folder               |
| histopt | optimize history                    |
| q       | quit yeet                           |
| w       | write changes without changing mode |
| wq      | write changes and quit yeet         |

## faq

### how fast is yeet

It utilizes the same mechanics like yazi (tokio i/o) without that many roundtrips
because of the underlying architecture. Thus, it should be equally fast. E.g. reading
a directory with 500k entries takes only a couple of seconds without blocking the
ui.

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

### yeet-keymap crate

This crate holds all key relevant features. The MessageResolver uses buffer
and tree to resolve possible messages, which follow the elm architecture to
modify the model.

tree uses the keymap to build a key tree structure. Thus, in keymap all
key combinations are mapped indirectly to messages.

conversion translates crossterm key events to the yeet-keymap
representation.
