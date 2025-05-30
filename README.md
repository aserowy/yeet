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

In navigation mode, all register interactions target the junk yard. The file
register holds all files which got yanked and the last nine trashes.

| keys      | action                                                      |
| --------- | ----------------------------------------------------------- |
| gh        | goto home directory                                         |
| gn        | go into normal mode                                         |
| h, l      | navigating the file tree                                    |
| p         | paste " from junk yard to current path                      |
| "p\<char> | paste register named \<char> from junk yard to current path |
| yp        | copy current selected path to system clipboard              |
| yy        | yank file to junk yard                                      |
| C-n, C-p  | navigate to (n)ext or (p) qfix entry                        |

### navigation and normal mode

| keys       | action                                                                                            |
| ---------- | ------------------------------------------------------------------------------------------------- |
| j, k       | navigating the current directory down/up                                                          |
| o, O       | add a new line and change to insert mode                                                          |
| I, A       | jump to line start/end and change to insert mode                                                  |
| dd         | go into normal and trash\* the current line                                                       |
| :          | change to command mode                                                                            |
| /          | change to search downward                                                                         |
| ?          | change to search upward                                                                           |
| n, N       | repeat last search in same/reverse direction                                                      |
| \<space>   | add or remove (toggle) current file to quick fix list                                             |
| q\<char>   | start recording a macro on register \<char>. Only letters [a-zA-Z] are allowed!                   |
| q          | while recording a macro, q finishes the recording and writes the input to the specified register. |
| @\<char>   | replay a recorded macro on register \<char>                                                       |
| @@         | replay the last played macro                                                                      |
| m\<char>   | set mark for current selection. Only letters [a-zA-Z] are allowed!                                |
| '\<char>   | jump to mark                                                                                      |
| zt, zz, zb | move viewport to start, center, bottom of cursor position                                         |
| C-u, C-d   | move viewport half screen up/down                                                                 |

\*trash: files are not deleted but moved to yeets cache folder to enable junk yard
interactions. Trashes get executed when leaving normal to navigation or saving the
current buffer. To delete the selected path completly, call command `:d!`.

### normal mode

In normal mode, all register interactions target the default register (equal to
`:reg` in nvim).

| keys               | action                                                           |
| ------------------ | ---------------------------------------------------------------- |
| h, l               | move cursor left/right                                           |
| 0, $               | move cursor to line start/end                                    |
| f\<char>, F\<char> | move cursor to next char forward/backward                        |
| t\<char>, T\<char> | move cursor before next char forward/backward                    |
| ;                  | repeat the last find motion with f or t.                         |
| ,                  | repeat the last find motion with f or t in reverse direction.    |
| e                  | move cursor to end of next word                                  |
| E                  | move cursor to end of next WORD                                  |
| ge                 | move cursor to end of next word backward                         |
| gE                 | move cursor to end of next WORD backward                         |
| w                  | move cursor to next word                                         |
| W                  | move cursor to next WORD                                         |
| b                  | move cursor to next word backward                                |
| B                  | move cursor to next WORD backward                                |
| i, a               | change to insert mode                                            |
| c\<motion>         | delete according to motion and change to insert mode             |
| d\<motion>         | delete according to motion                                       |
| s                  | delete char on cursor and change to insert mode                  |
| x                  | delete char on cursor                                            |
| .                  | repeat last modification. Key sequence is stored in '.' register |

## commands

> [!NOTE]
> all paths for path arguments can be absolute or relative to the current path shown!

| :                           | action                                                                                                                                                                                                                 |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| cfirst                      | navigates to first entry in quick fix list                                                                                                                                                                             |
| cl                          | list all quick fix entries and highlights the current path                                                                                                                                                             |
| clearcl \<empty> or \<path> | clears qfix completely if empty or clears all entries in the given folder.                                                                                                                                             |
| cn, cN                      | navigates to next/previous path in quick fix list                                                                                                                                                                      |
| cdo \<command>              | navigates to each entry in the quick fix list and executes the given command.<br>Cdo starts with the first entry and iterates over the given order. Thus, the list order is important! Non existing paths get ignored. |
| cp \<path> or '\<mark>      | copies the selected file to the target directory. The directory must exist without a file with the same name like the source                                                                                           |
| d!                          | delete selected file/directory                                                                                                                                                                                         |
| delm \<chars>               | delete current and cached marks. Every char represents one mark. ':delm AdfR', ':delm a d f R', and ':delm F' are all valid commands. Whitespaces are ignored.                                                         |
| delt \<task_id>             | stop a task with the given id. The id can be found by listing tasks with `tl`                                                                                                                                          |
| e!                          | reload current folder                                                                                                                                                                                                  |
| fd \<params for fd>         | uses (fd)[https://github.com/sharkdp/] to populate qfix. \<params for fd> are passed through to fd. Yeet sets the following params by default: --color never --absolute-path --base-directory current_path             |
| invertcl                    | inverts the cl selection in current folder                                                                                                                                                                             |
| junk                        | list junk yard contents                                                                                                                                                                                                |
| marks                       | list all given marks                                                                                                                                                                                                   |
| mv \<path> or '\<mark>      | moves the selected file to the target. The directory must exist without a file with the same name like the source                                                                                                      |
| noh                         | remove search highlights                                                                                                                                                                                               |
| q                           | quit yeet                                                                                                                                                                                                              |
| q!                          | force tasks to stop and quit yeet                                                                                                                                                                                      |
| reg                         | print all register entries                                                                                                                                                                                             |
| tl                          | list all currently running tasks                                                                                                                                                                                       |
| w                           | write changes without changing mode                                                                                                                                                                                    |
| wq                          | write changes and quit yeet                                                                                                                                                                                            |
| z \<target for z>           | jump to paths with zoxide like in your terminal. `:z foo` will execute zoxide to jump to the given directory |

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
