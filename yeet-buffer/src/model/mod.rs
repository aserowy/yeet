use std::fmt::{self, Display};

use crate::message::CursorDirection;

use ansi::Ansi;
use undo::{BufferChanged, Undo};

pub mod ansi;
pub mod undo;
pub mod viewport;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
    Command(CommandMode),
    Insert,
    #[default]
    Navigation,
    Normal,
}

impl Mode {
    pub fn is_command(&self) -> bool {
        matches!(self, Mode::Command(_))
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Mode::Command(_) => "command".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Navigation => "navigation".to_string(),
            Mode::Normal => "normal".to_string(),
        };

        write!(f, "{}", content)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CommandMode {
    Command,
    PrintMultiline,
    Search(SearchDirection),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum SearchDirection {
    #[default]
    Down,
    Up,
}

#[derive(Default)]
pub struct TextBuffer {
    pub last_find: Option<CursorDirection>,
    pub lines: Vec<BufferLine>,
    pub(crate) undo: Undo,
}

impl TextBuffer {
    pub fn has_unsaved_changes(&self) -> bool {
        !self.undo.get_uncommited_changes().is_empty()
    }

    pub fn uncommitted_changes(&self) -> Vec<BufferChanged> {
        self.undo.get_uncommited_changes()
    }

    pub fn from_lines(lines: Vec<BufferLine>) -> Self {
        Self {
            last_find: None,
            lines,
            undo: Undo::default(),
        }
    }

    pub fn revert_unsaved_changes(&mut self) {
        let changes = self.undo.get_uncommited_changes();
        if !changes.is_empty() {
            for change in changes.iter().rev() {
                match change {
                    BufferChanged::LineAdded(idx, _) => {
                        if *idx < self.lines.len() {
                            self.lines.remove(*idx);
                        }
                    }
                    BufferChanged::LineRemoved(idx, content) => {
                        let line = BufferLine {
                            content: content.clone(),
                            ..Default::default()
                        };
                        if *idx <= self.lines.len() {
                            self.lines.insert(*idx, line);
                        } else {
                            self.lines.push(line);
                        }
                    }
                    BufferChanged::Content(idx, old, _) => {
                        if let Some(line) = self.lines.get_mut(*idx) {
                            line.content = old.clone();
                        }
                    }
                }
            }
        }
        self.undo.reset_to_last_save();
    }
}

impl fmt::Debug for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("last_find", &self.last_find)
            .field("lines", &self.lines)
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub horizontal_index: CursorPosition,
    pub vertical_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CursorPosition {
    Absolute { current: usize, expanded: usize },
    End,
    None,
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BufferLine {
    pub prefix: Option<String>,
    pub content: Ansi,
    pub search_char_position: Option<Vec<(usize, usize)>>,
    pub signs: Vec<Sign>,
    /// Icon glyph set by plugin mutation hooks. Rendered in the icon-column prefix segment.
    pub icon: Option<String>,
    /// ANSI foreground color string for icon glyph and filename text, set by plugin mutation hooks.
    pub icon_style: Option<String>,
}

impl BufferLine {
    pub fn from(content: &str) -> Self {
        Self {
            content: Ansi::new(content),
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn len(&self) -> usize {
        self.content.count_chars()
    }
}

pub type SignIdentifier = &'static str;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sign {
    pub id: SignIdentifier,
    pub content: char,
    pub priority: usize,
    pub style: String,
}

#[derive(Clone, PartialEq)]
pub enum BufferResult {
    Changes(Vec<BufferChanged>),
    CursorPositionChanged,
    FindScopeChanged(CursorDirection),
}
