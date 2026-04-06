# Keybindings

This page provides a quick reference of all keybindings organized by category. For detailed descriptions of each mode and its keybindings, see `:help modes`.

## Navigation

### `h`

Move to the parent directory in Navigation mode, or move cursor left in Normal mode. The behavior depends on which mode is currently active.

### `j`

Move cursor down by one entry or line. In Navigation mode this selects the next directory entry; in Normal mode it moves to the next text line.

### `k`

Move cursor up by one entry or line. In Navigation mode this selects the previous directory entry; in Normal mode it moves to the previous text line.

### `l`

Enter the selected directory or open a file in Navigation mode, or move cursor right in Normal mode. The behavior depends on which mode is currently active.

### `gg`

Jump to the first entry or line in the buffer. This moves the cursor to the very top of the directory listing or text buffer.

### `G`

Jump to the last entry or line in the buffer. This moves the cursor to the very bottom of the directory listing or text buffer.

### `Ctrl-d`

Scroll down by half a screen height. The cursor moves down by the same amount, keeping its relative position in the viewport.

### `Ctrl-u`

Scroll up by half a screen height. The cursor moves up by the same amount, keeping its relative position in the viewport.

## Window Navigation

### `Ctrl-h`

Move focus to the split window on the left. If there is no window to the left, focus stays on the current window.

### `Ctrl-j`

Move focus to the split window below. If there is no window below, focus stays on the current window.

### `Ctrl-k`

Move focus to the split window above. If there is no window above, focus stays on the current window.

### `Ctrl-l`

Move focus to the split window on the right. If there is no window to the right, focus stays on the current window.

## Viewport

### `zt`

Reposition the viewport so the cursor line is at the top of the screen. The cursor position itself does not change; only the viewport scrolls.

### `zz`

Reposition the viewport so the cursor line is centered on the screen. The cursor position itself does not change; only the viewport scrolls.

### `zb`

Reposition the viewport so the cursor line is at the bottom of the screen. The cursor position itself does not change; only the viewport scrolls.

## File Operations

### `Enter`

Open the selected file with the system default application or enter the selected directory. In a quickfix window, this navigates to the entry's path in the nearest directory window.

### `o`

Add a new empty line below the cursor and enter Insert mode. In a directory buffer this creates a new entry below the current selection.

### `O`

Add a new empty line above the cursor and enter Insert mode. In a directory buffer this creates a new entry above the current selection.

### `I`

Move the cursor to the beginning of the line and enter Insert mode. This is a shortcut for `0i` and is useful for prepending text to a filename.

### `A`

Move the cursor to the end of the line and enter Insert mode. This is a shortcut for `$a` and is useful for appending text to a filename.

### `dd`

Trash the current entry and enter Normal mode. The file is moved to yeet's junk yard cache, not permanently deleted. Use `:d!` for permanent deletion.

### `yy`

Yank the selected file or directory into the junk yard. The entry is stored in the default register and can be pasted with `p`.

### `p`

Paste the default junk yard register contents to the current directory. The file is restored from yeet's cache to the currently shown path.

### `yp`

Copy the absolute path of the currently selected entry to the system clipboard. This is useful for pasting the path into other applications.

## Quickfix

### `Space`

Toggle the currently selected file in the quickfix list. If the file is not in the list it is added; if already present it is removed.

### `Ctrl-n`

Navigate to the next quickfix entry. This moves the directory view to the path of the next entry in the quickfix list.

### `Ctrl-p`

Navigate to the previous quickfix entry. This moves the directory view to the path of the previous entry in the quickfix list.

## Marks

### `m`

Set a mark at the current cursor position. Usage: `m<char>` stores the current path and position under the named mark. Only letters `[a-zA-Z]` are valid.

### `'`

Jump to a previously set mark. Usage: `'<char>` navigates to the path and position stored under the named mark.

## Search

### `/`

Start a forward search from the current position. A search prompt appears at the bottom of the screen; type a pattern and press Enter to find the next match.

### `?`

Start a backward search from the current position. A search prompt appears at the bottom of the screen; type a pattern and press Enter to find the previous match.

### `n`

Jump to the next match of the last search pattern. The direction follows the original search — forward for `/` and backward for `?`.

### `N`

Jump to the previous match of the last search pattern. The direction is reversed from the original — backward for `/` and forward for `?`.

## Tabs

### `gt`

Switch to the next tab in the tab bar. If the current tab is the last one, this wraps around to the first tab.

### `gT`

Switch to the previous tab in the tab bar. If the current tab is the first one, this wraps around to the last tab.

## Mode Switching

### `gn`

Enter Normal mode from Navigation mode. This allows text editing operations such as renaming files directly in the buffer.

### `:`

Enter Command mode from Navigation or Normal mode. A command prompt appears at the bottom of the screen. See `:help commands` for available commands.

### `Esc`

Return to the next higher-level mode. In Insert mode this enters Normal mode; in Normal mode this enters Navigation mode. In Command mode this restores the previous mode.

## Registers and Macros

### `q`

Start or stop recording a macro. Usage: `q<char>` begins recording keystrokes to the named register; pressing `q` again stops and saves the macro. Only letters `[a-zA-Z]` are valid register names.

### `@`

Replay a recorded macro from a named register. Usage: `@<char>` plays back the stored keystrokes. Use `@@` to replay the last played macro.

### `"`

Select a register for the next operation. Usage: `"<char>` sets the active register. In Navigation mode this targets junk yard registers; in Normal mode it targets text registers.

## Normal Mode Motions

For the full list of Normal mode text motions (`w`, `e`, `b`, `f`, `t`, `0`, `$`, etc.) and edit operations (`c`, `d`, `s`, `x`, `.`), see `:help modes` under the Normal section.

## Splits

### `Ctrl-w Ctrl-s`

Create a horizontal split of the current directory view. The new pane appears below showing the same directory path.

### `Ctrl-w Ctrl-v`

Create a vertical split of the current directory view. The new pane appears to the right showing the same directory path.
