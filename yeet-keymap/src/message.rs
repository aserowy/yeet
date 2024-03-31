use std::path::PathBuf;

use regex::Regex;
use yeet_buffer::{
    message::{BufferMessage, CursorDirection, TextModification},
    model::Mode,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub expects: Option<NextBindingKind>,
    pub force: Option<Mode>,
    pub kind: BindingKind,
    pub repeat: Option<usize>,
    pub repeatable: bool,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            expects: None,
            force: None,
            kind: BindingKind::default(),
            repeat: None,
            repeatable: true,
        }
    }
}

impl Binding {
    pub fn from_motion(motion: CursorDirection) -> Self {
        Self {
            kind: BindingKind::Motion(motion),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum NextBindingKind {
    Motion,
    Raw(Option<Regex>),
}

impl PartialEq for NextBindingKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Motion, Self::Motion) => true,
            (Self::Raw(self_reg), Self::Raw(reg)) => match (self_reg, reg) {
                (Some(self_reg), Some(reg)) => self_reg.as_str() == reg.as_str(),
                (None, None) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum BindingKind {
    Message(Message),
    Motion(CursorDirection),
    #[default]
    None,
    Raw(char),
    Repeat,
    RepeatOrMotion(CursorDirection),
    Modification(TextModification),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Buffer(BufferMessage),
    DeleteMarks(Vec<char>),
    ClearSearchHighlight,
    EnumerationChanged(PathBuf, Vec<(ContentKind, String)>, Option<String>),
    EnumerationFinished(PathBuf, Option<String>),
    Error(String),
    ExecuteCommand,
    ExecuteCommandString(String),
    KeySequenceChanged(String),
    NavigateToMark(char),
    NavigateToParent,
    NavigateToPath(PathBuf),
    NavigateToPathAsPreview(PathBuf),
    NavigateToSelected,
    OpenSelected,
    PasteFromJunkYard(char),
    PathRemoved(PathBuf),
    PathsAdded(Vec<PathBuf>),
    PreviewLoaded(PathBuf, Vec<String>),
    Print(Vec<PrintContent>),
    Rerender,
    Resize(u16, u16),
    SetMark(char),
    ToggleQuickFix,
    Quit,
    // TODO: yank to junk with motion
    YankToJunkYard(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintContent {
    Error(String),
    Default(String),
    Information(String),
}
