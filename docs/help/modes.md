# Modes

Yeet uses four modes inspired by vim. Each mode determines which keybindings are active and how input is interpreted.

## Mode Transitions

### `Esc`

In every mode, pressing `Esc` transitions to the next higher-level mode. The mode hierarchy from lowest to highest is: Insert, Normal, Navigation. Pressing `Esc` in Insert mode enters Normal mode, and pressing `Esc` in Normal mode enters Navigation mode.

### `Command mode exception`

Command mode does not follow the standard hierarchy. Leaving command mode restores whichever mode was active before entering it, rather than moving up the hierarchy.

### `Filesystem persistence`

Changes made in Normal and Insert mode behave like unsaved buffer edits. They are not written to disk until you explicitly save with `:w` or transition from Normal mode to Navigation mode, which triggers an automatic save.

## Navigation

### `Navigation`

Navigation mode is the default mode and is used for browsing the filesystem. In this mode, `h` and `l` move between parent and child directories, `j` and `k` move the cursor up and down, and `Enter` opens the selected file or directory.

### `Register targeting`

In Navigation mode, all register interactions target the junk yard. The default register `"` holds yanked files and the last nine trashed entries, similar to vim's numbered registers but for filesystem operations.

### `Enter`

Opens the selected file or enters the selected directory. If the selection is a file, it is opened with the system default application via `xdg-open` on Linux or the platform equivalent.

### `gh`

Navigates to the user's home directory. This is equivalent to `cd ~` in a shell and moves the current directory view to the home path.

### `gn`

Enters Normal mode from Navigation mode. This allows text editing operations such as renaming files by modifying directory entry names directly in the buffer.

### `gt`

Switches to the next tab in the tab bar. If the current tab is the last one, this wraps around to the first tab.

### `gT`

Switches to the previous tab in the tab bar. If the current tab is the first one, this wraps around to the last tab.

### `p`

Pastes the contents of the default junk yard register to the current directory. The pasted file or directory is restored from the junk yard cache to the path currently shown in the directory view.

### `"p`

Pastes from a named junk yard register. Usage: `"p<char>` where `<char>` selects the register. Only letters `[a-zA-Z]` and digits `[0-9]` are valid register names.

### `yp`

Copies the absolute path of the currently selected entry to the system clipboard. This is useful for pasting the path into other applications or terminal commands.

### `yy`

Yanks the selected file or directory into the junk yard. The entry is moved to yeet's cache folder and stored in the default register, making it available for pasting with `p`.

### `C-n`

Navigates to the next quickfix entry. This moves the directory view to the path of the next entry in the quickfix list, advancing the quickfix index by one.

### `C-p`

Navigates to the previous quickfix entry. This moves the directory view to the path of the previous entry in the quickfix list, decrementing the quickfix index by one.

### `C-w C-s`

Creates a horizontal split of the current directory view. The new pane appears below the current one, showing the same directory path.

### `C-w C-v`

Creates a vertical split of the current directory view. The new pane appears to the right of the current one, showing the same directory path.

## Normal

### `Normal`

Normal mode is for text editing when renaming files or directories. In this mode, register interactions target the default register (equivalent to `:reg` in vim), not the junk yard.

### `h (Normal)`

Moves the cursor one character to the left within the current line. If the cursor is already at the beginning of the line, it stays in place.

### `l (Normal)`

Moves the cursor one character to the right within the current line. If the cursor is already at the end of the line, it stays in place.

### `0`

Moves the cursor to the beginning of the current line. This is equivalent to the `Home` key and places the cursor at column zero.

### `$`

Moves the cursor to the end of the current line. This moves the cursor to the last character position on the line.

### `f`

Finds and moves the cursor to the next occurrence of a character forward. Usage: `f<char>` — the cursor lands on the matching character.

### `F`

Finds and moves the cursor to the next occurrence of a character backward. Usage: `F<char>` — the cursor lands on the matching character searching left from the current position.

### `t`

Moves the cursor to just before the next occurrence of a character forward. Usage: `t<char>` — the cursor lands one position before the matching character.

### `T`

Moves the cursor to just after the next occurrence of a character backward. Usage: `T<char>` — the cursor lands one position after the matching character searching left.

### `;`

Repeats the last `f`, `F`, `t`, or `T` motion in the same direction. This allows quickly jumping to the next occurrence without retyping the character.

### `,`

Repeats the last `f`, `F`, `t`, or `T` motion in the reverse direction. This is the complement to `;` and searches in the opposite direction of the original motion.

### `w`

Moves the cursor to the beginning of the next word. Words are delimited by non-alphanumeric characters, following vim's `iskeyword` conventions.

### `W`

Moves the cursor to the beginning of the next WORD. A WORD is delimited only by whitespace, so punctuation and special characters are included as part of the WORD.

### `b`

Moves the cursor to the beginning of the previous word. This moves backward by word boundaries, the reverse of `w`.

### `B`

Moves the cursor to the beginning of the previous WORD. This moves backward by whitespace-delimited boundaries, the reverse of `W`.

### `e`

Moves the cursor to the end of the next word. The cursor lands on the last character of the current or next word.

### `E`

Moves the cursor to the end of the next WORD. The cursor lands on the last character before the next whitespace.

### `ge`

Moves the cursor to the end of the previous word. This is the backward equivalent of `e`.

### `gE`

Moves the cursor to the end of the previous WORD. This is the backward equivalent of `E`.

### `i`

Enters Insert mode at the current cursor position. Text typed after entering insert mode is inserted before the character under the cursor.

### `a`

Enters Insert mode after the current cursor position. Text typed after entering insert mode is inserted after the character under the cursor.

### `c`

Deletes text according to a motion and enters Insert mode. Usage: `c<motion>` — the text covered by the motion is removed and the mode changes to Insert for replacement.

### `d (Normal)`

Deletes text according to a motion without entering Insert mode. Usage: `d<motion>` — the text covered by the motion is removed and stored in the default register.

### `s`

Deletes the character under the cursor and enters Insert mode. This is a shortcut for `cl` — it removes one character and immediately allows typing a replacement.

### `x`

Deletes the character under the cursor without entering Insert mode. This is a shortcut for `dl` — the deleted character is stored in the default register.

### `.`

Repeats the last text modification. The key sequence of the last edit is stored in the `.` register and replayed at the current cursor position.

## Shared

### `j`

Moves the cursor down by one line in the current buffer. In Navigation mode this selects the next directory entry; in Normal mode it moves to the next line of the text buffer.

### `k`

Moves the cursor up by one line in the current buffer. In Navigation mode this selects the previous directory entry; in Normal mode it moves to the previous line of the text buffer.

### `gg`

Jumps to the first entry or line in the buffer. This moves the cursor to the very top of the directory listing or text buffer.

### `G`

Jumps to the last entry or line in the buffer. This moves the cursor to the very bottom of the directory listing or text buffer.

### `o`

Adds a new empty line below the cursor and enters Insert mode. In a directory buffer this creates a new entry below the current selection.

### `O`

Adds a new empty line above the cursor and enters Insert mode. In a directory buffer this creates a new entry above the current selection.

### `I`

Moves the cursor to the beginning of the line and enters Insert mode. This is a shortcut for `0i` and is useful for prepending text to a filename.

### `A`

Moves the cursor to the end of the line and enters Insert mode. This is a shortcut for `$a` and is useful for appending text to a filename.

### `dd`

Trashes the current line and enters Normal mode. In a directory buffer this moves the selected file to yeet's junk yard cache rather than permanently deleting it. Use `:d!` for permanent deletion.

### `:`

Enters Command mode from Navigation or Normal mode. A command prompt appears at the bottom of the screen where you can type commands like `:w`, `:q`, or `:help`.

### `/`

Starts a forward search from the current position. A search prompt appears at the bottom of the screen; type a pattern and press Enter to find the next match.

### `?`

Starts a backward search from the current position. A search prompt appears at the bottom of the screen; type a pattern and press Enter to find the previous match.

### `n`

Jumps to the next match of the last search pattern. The search direction follows the original search — forward for `/` and backward for `?`.

### `N`

Jumps to the previous match of the last search pattern. The search direction is reversed from the original — backward for `/` and forward for `?`.

### `Space`

Toggles the currently selected file in the quickfix list. If the file is not in the quickfix list it is added; if it is already present it is removed.

### `q (macro)`

Starts or stops recording a macro. Usage: `q<char>` begins recording keystrokes to the named register; pressing `q` again while recording stops and saves the macro. Only letters `[a-zA-Z]` are valid register names.

### `@`

Replays a recorded macro. Usage: `@<char>` plays back the keystrokes stored in the named register. Use `@@` to replay the last played macro.

### `m`

Sets a mark at the current cursor position. Usage: `m<char>` stores the current path and position under the named mark. Only letters `[a-zA-Z]` are valid mark names.

### `'`

Jumps to a previously set mark. Usage: `'<char>` navigates to the path and position stored under the named mark.

### `zt`

Repositions the viewport so the cursor line is at the top of the screen. The cursor position does not change; only the viewport scrolls.

### `zz`

Repositions the viewport so the cursor line is centered on the screen. The cursor position does not change; only the viewport scrolls.

### `zb`

Repositions the viewport so the cursor line is at the bottom of the screen. The cursor position does not change; only the viewport scrolls.

### `C-u`

Scrolls the viewport up by half a screen height. The cursor moves up by the same amount, keeping its relative position in the viewport.

### `C-d`

Scrolls the viewport down by half a screen height. The cursor moves down by the same amount, keeping its relative position in the viewport.

### `C-h`

Moves focus to the split window on the left. If there is no window to the left, focus stays on the current window.

### `C-j`

Moves focus to the split window below. If there is no window below, focus stays on the current window.

### `C-k`

Moves focus to the split window above. If there is no window above, focus stays on the current window.

### `C-l`

Moves focus to the split window on the right. If there is no window to the right, focus stays on the current window.

## Insert

### `Insert`

Insert mode is active when editing text in a rename or new entry operation. Characters typed in this mode are inserted at the cursor position. Press `Esc` to return to Normal mode.

## Command

### `Command`

Command mode is entered by pressing `:` and displays a prompt at the bottom of the screen. Type a command and press `Enter` to execute it, or press `Esc` to cancel and return to the previous mode. See `:help commands` for all available commands.
