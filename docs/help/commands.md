# Commands

All commands are entered in command mode by pressing `:`.

## File Operations

### `w`

Save the current buffer.

### `e!`

Refresh the current buffer from disk.

### `cp`

Copy a path. Usage: `:cp <target>`

### `mv`

Move or rename a path. Usage: `:mv <target>`

### `d!`

Delete the path under the cursor (irreversible).

## Quit

### `q`

Close the focused window. If the last window, quit the application.
Fails if there are unsaved changes.

### `q!`

Force close the focused window, discarding unsaved changes.

### `qa`

Quit the application. Fails if there are unsaved changes.

### `qa!`

Force quit the application, discarding all unsaved changes.

### `wq`

Save and close the focused window.

## Splits

### `split`

Open a horizontal split. Usage: `:split <path>`

### `vsplit`

Open a vertical split. Usage: `:vsplit <path>`

## Tabs

### `tabnew`

Create a new tab.

### `tabc`

Close the current tab.

### `tabc!`

Force close the current tab.

### `tabo`

Close all other tabs.

### `tabo!`

Force close all other tabs.

### `tabfir`

Switch to the first tab.

### `tabl`

Switch to the last tab.

### `tabn`

Switch to the next tab.

### `tabp`

Switch to the previous tab.

### `tabs`

List all open tabs.

## Search

### `fd`

Execute fd in the current directory. Usage: `:fd <params>`

### `rg`

Execute ripgrep in the current directory. Usage: `:rg <params>`

### `noh`

Clear search highlighting.

## Quickfix

### `copen`

Open the quickfix window.

### `cl`

List quickfix entries.

### `cn`

Jump to the next quickfix entry.

### `cN`

Jump to the previous quickfix entry.

### `cfirst`

Jump to the first quickfix entry.

### `clearcl`

Clear the quickfix list. Usage: `:clearcl [path]`

### `invertcl`

Invert the quickfix selection in the current directory.

## Tasks

### `topen`

Open the tasks window.

### `tl`

List running tasks.

### `delt`

Delete a task by ID. Usage: `:delt <id>`

## Other

### `cdo`

Execute a command on each quickfix entry. Usage: `:cdo <command>`

### `marks`

Display all marks.

### `delm`

Delete marks. Usage: `:delm <marks>`

### `reg`

Display registers.

### `junk`

Display the junkyard (trash).

### `z`

Execute zoxide. Usage: `:z <params>`

### `help`

Open the help system. Usage: `:help [topic]`
