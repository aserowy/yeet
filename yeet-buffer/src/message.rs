use std::cmp::Ordering;

use crate::model::{BufferLine, Mode};

#[derive(Clone, Eq, PartialEq)]
pub enum BufferMessage {
    ChangeMode(Mode, Mode),
    Modification(usize, TextModification),
    MoveCursor(usize, CursorDirection),
    MoveViewPort(ViewPortDirection),
    RemoveLine(usize),
    ResetCursor,
    SaveBuffer,
    SetContent(Vec<BufferLine>),
    SetCursorToLineContent(String),
    SortContent(fn(&BufferLine, &BufferLine) -> Ordering),
    UpdateViewPortByCursor,
}

impl std::fmt::Debug for BufferMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferMessage::ChangeMode(from, to) => {
                f.debug_tuple("ChangeMode").field(from).field(to).finish()
            }
            BufferMessage::Modification(index, modification) => f
                .debug_tuple("Modification")
                .field(index)
                .field(modification)
                .finish(),
            BufferMessage::MoveCursor(index, direction) => f
                .debug_tuple("MoveCursor")
                .field(index)
                .field(direction)
                .finish(),
            BufferMessage::MoveViewPort(direction) => {
                f.debug_tuple("MoveViewPort").field(direction).finish()
            }
            BufferMessage::RemoveLine(index) => f.debug_tuple("RemoveLine").field(index).finish(),
            BufferMessage::ResetCursor => f.debug_tuple("ResetCursor").finish(),
            BufferMessage::SaveBuffer => f.debug_tuple("SaveBuffer").finish(),
            BufferMessage::SetContent(_) => f.debug_tuple("SetContent").finish(),
            BufferMessage::SetCursorToLineContent(content) => f
                .debug_tuple("SetCursorToLineContent")
                .field(content)
                .finish(),
            BufferMessage::SortContent(_) => f
                .debug_tuple("SortContent")
                .field(&"fn(&BufferLine, &BufferLine) -> Ordering")
                .finish(),
            BufferMessage::UpdateViewPortByCursor => {
                f.debug_tuple("UpdateViewPortByCursor").finish()
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TextModification {
    DeleteLine,
    DeleteMotion(usize, CursorDirection),
    Insert(String),
    InsertLineBreak,
    InsertNewLine(LineDirection),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LineDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CursorDirection {
    Bottom,
    Down,
    FindBackward(char),
    FindForward(char),
    LastFindBackward,
    LastFindForward,
    Left,
    LineEnd,
    LineStart,
    Right,
    Search(Search),
    TillBackward(char),
    TillForward(char),
    Top,
    Up,
    WordEndBackward,
    WordEndForward,
    WordStartBackward,
    WordStartForward,
    WordUpperEndBackward,
    WordUpperEndForward,
    WordUpperStartBackward,
    WordUpperStartForward,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Search {
    #[default]
    Next,
    Previous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewPortDirection {
    BottomOnCursor,
    CenterOnCursor,
    HalfPageDown,
    HalfPageUp,
    TopOnCursor,
}
