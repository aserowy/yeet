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
    pub toggle: Option<(String, BindingKind)>,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            expects: None,
            force: None,
            kind: BindingKind::default(),
            repeat: None,
            repeatable: true,
            toggle: None,
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
    Message(KeymapMessage),
    Motion(CursorDirection),
    #[default]
    None,
    Raw(char),
    Repeat,
    RepeatOrMotion(CursorDirection),
    Modification(TextModification),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeySequence {
    Completed(String),
    Changed(String),
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeymapMessage {
    Buffer(BufferMessage),
    ClearSearchHighlight,
    DeleteMarks(Vec<char>),
    ExecuteCommand,
    ExecuteCommandString(String),
    ExecuteKeySequence(String),
    ExecuteRegister(char),
    LeaveCommandMode,
    NavigateToMark(char),
    NavigateToParent,
    NavigateToPath(PathBuf),
    NavigateToPathAsPreview(PathBuf),
    NavigateToSelected,
    OpenSelected,
    PasteFromJunkYard(char),
    Print(Vec<PrintContent>),
    ReplayMacro(char),
    SetMark(char),
    StartMacro(char),
    StopMacro,
    ToggleQuickFix,
    Quit,
    YankPathToClipboard,
    // TODO: yank to junk with motion
    YankToJunkYard(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintContent {
    Error(String),
    Default(String),
    Information(String),
}
