use std::cmp::Ordering;

use crate::model::{BufferLine, Mode};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BufferMessage {
    // TODO: Yank & Paste in normal mode into reg
    ChangeMode(Mode, Mode),
    Modification(usize, TextModification),
    MoveCursor(usize, CursorDirection),
    MoveViewPort(ViewPortDirection),
    RemoveLine(usize),
    ResetCursor,
    SaveBuffer(Option<usize>),
    SetContent(Vec<BufferLine>),
    SetCursorToLineContent(String),
    SortContent(fn(&BufferLine, &BufferLine) -> Ordering),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TextModification {
    DeleteLine,
    DeleteMotion(usize, CursorDirection),
    Insert(String),
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
    Left,
    LineEnd,
    LineStart,
    Right,
    Search(bool),
    TillBackward(char),
    TillForward(char),
    Top,
    Up,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum SearchDirection {
    Up,
    #[default]
    Down,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewPortDirection {
    BottomOnCursor,
    CenterOnCursor,
    HalfPageDown,
    HalfPageUp,
    TopOnCursor,
}
