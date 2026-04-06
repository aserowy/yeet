# Commands

All commands are entered in command mode by pressing `:`. Path arguments can be absolute or relative to the current directory shown. Mark references like `'a` can be used in place of paths where noted.

## File Operations

### `w`

Save the current buffer to disk without changing mode. In a directory buffer, this writes all pending renames and new entries to the filesystem.

### `e!`

Refresh the current buffer from disk, discarding any unsaved changes. This is useful when external tools have modified files and you want to reload the directory listing.

### `cp`

Copy the selected file or directory to a target path. Usage: `:cp <path>` or `:cp '<mark>`. The target directory must exist and must not already contain a file with the same name as the source.

### `mv`

Move or rename the selected file or directory to a target path. Usage: `:mv <path>` or `:mv '<mark>`. The target directory must exist and must not already contain a file with the same name as the source.

### `d!`

Permanently delete the file or directory under the cursor. This is irreversible and bypasses the junk yard — use `dd` in navigation mode if you want recoverable deletion via the junk yard.

## Quit

### `q`

Close the focused window, or quit the application if it is the last window. This command fails if there are unsaved changes; use `:q!` to discard changes or `:w` to save first.

### `q!`

Force close the focused window, discarding all unsaved changes. Running tasks are also stopped. If this is the last window, the application quits.

### `qa`

Quit the application by closing all windows and tabs. This command fails if any buffer has unsaved changes; use `:qa!` to discard all changes.

### `qa!`

Force quit the application, discarding all unsaved changes across all windows and tabs. Running tasks are stopped before exiting.

### `wq`

Save the current buffer and close the focused window. This is equivalent to running `:w` followed by `:q`, and quits the application if it is the last window.

## Splits

### `split`

Open a horizontal split with a new pane below the current one. Usage: `:split <path>` opens the given path in the new pane, or `:split` with no arguments opens the current directory. Path can be absolute, relative, or a mark reference.

### `vsplit`

Open a vertical split with a new pane to the right of the current one. Usage: `:vsplit <path>` opens the given path in the new pane, or `:vsplit` with no arguments opens the current directory. Path can be absolute, relative, or a mark reference.

## Tabs

### `tabnew`

Create a new tab and switch to it. The new tab opens with a directory view of the current path.

### `tabc`

Close the current tab. This command fails if any buffer in the tab has unsaved changes; use `:tabc!` to force close.

### `tabc!`

Force close the current tab, discarding all unsaved changes. All buffers in the tab are reset before closing.

### `tabo`

Close all tabs except the current one. This command fails if any other tab has unsaved changes; use `:tabo!` to force close.

### `tabo!`

Force close all tabs except the current one, discarding all unsaved changes. All buffers in the closed tabs are reset.

### `tabfir`

Switch to the first tab in the tab bar. If already on the first tab, this has no effect.

### `tabl`

Switch to the last tab in the tab bar. If already on the last tab, this has no effect.

### `tabn`

Switch to the next tab in the tab bar. This is equivalent to the `gt` keybinding and wraps around to the first tab if on the last.

### `tabp`

Switch to the previous tab in the tab bar. This is equivalent to the `gT` keybinding and wraps around to the last tab if on the first.

### `tabs`

List all open tabs with their index and current path. The output is shown in the command line area.

## Search

### `fd`

Execute `fd` in the current directory and populate the quickfix list with results. Usage: `:fd <params>` — parameters are passed directly to fd. Yeet automatically adds `--color never --absolute-path --base-directory <current_path>`.

### `rg`

Execute `ripgrep` in the current directory and populate the quickfix list with matching file paths. Usage: `:rg <params>` — parameters are passed directly to rg. Yeet automatically adds `--color never --files-with-matches <params> <current_path>`.

### `noh`

Clear all search highlighting from the current buffer. This removes the background color from matches found by `/` or `?` search without affecting the search pattern itself.

## Quickfix

### `copen`

Open the quickfix window in a horizontal split below the current pane. The quickfix window shows all entries with their index and path. Use `dd` to remove entries and `Enter` to navigate to an entry.

### `cl`

List all quickfix entries in the command line area, highlighting the current entry. This provides a quick overview without opening the full quickfix window.

### `cn`

Jump to the next quickfix entry in the list. The directory view navigates to the path of the next entry and the quickfix index advances by one.

### `cN`

Jump to the previous quickfix entry in the list. The directory view navigates to the path of the previous entry and the quickfix index decrements by one.

### `cfirst`

Jump to the first quickfix entry in the list. This resets the quickfix index to zero and navigates to the first entry's path.

### `clearcl`

Clear entries from the quickfix list. Usage: `:clearcl` clears all entries, or `:clearcl <path>` clears only entries within the given directory.

### `invertcl`

Invert the quickfix selection in the current directory. Files that are in the quickfix list are removed, and files that are not in the list are added.

## Tasks

### `topen`

Open the tasks window in a horizontal split below the current pane. The tasks window shows all running background tasks. Use `dd` to stop and remove a selected task.

### `tl`

List all currently running tasks in the command line area. Each task is shown with its ID and description for reference when using `:delt`.

### `delt`

Stop and remove a task by its ID. Usage: `:delt <id>`. The task ID can be found by running `:tl` or by viewing the tasks window with `:topen`.

## Other

### `cdo`

Execute a command on each quickfix entry in order. Usage: `:cdo <command>`. Yeet navigates to each entry's path and runs the given command. Non-existing paths are skipped. The list order determines execution order.

### `marks`

Display all currently set marks with their names and paths. Each mark is shown as a letter-to-path mapping in the command line area.

### `delm`

Delete one or more marks by name. Usage: `:delm <chars>` where each character is a mark to delete. Whitespace is ignored, so `:delm AdfR`, `:delm a d f R`, and `:delm F` are all valid.

### `reg`

Display the contents of all registers in the command line area. This shows both text registers (used in Normal mode) and junk yard registers (used in Navigation mode).

### `junk`

Display the contents of the junk yard in the command line area. The junk yard holds yanked files and the last nine trashed entries, which can be restored with `p`.

### `set wrap`

Enable word wrapping on the current window. Long lines are broken at word boundaries to fit within the viewport width. For directory windows, wrapping is applied to all three panes (parent, current, preview). For split windows, only the focused pane is affected.

### `set nowrap`

Disable word wrapping on the current window. Lines that exceed the viewport width are shown with horizontal scrolling instead. For directory windows, wrapping is disabled on all three panes. For split windows, only the focused pane is affected.

### `z`

Navigate to a directory using zoxide fuzzy matching. Usage: `:z <query>`. Yeet passes the query to zoxide and jumps to the resolved directory, the same way `z` works in your shell.

### `help`

Open the help system in a horizontal split. Usage: `:help` opens the index page, or `:help <topic>` opens the page matching the topic. Topic search is case-insensitive and matches page names, section headings, and entry identifiers.
