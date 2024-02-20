use std::path::PathBuf;

use yeet_keymap::message::Mode;

use crate::task::Task;

#[derive(Clone, Debug, PartialEq)]
pub enum RenderAction {
    PreView(PreViewAction),
    Post(PostAction),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreViewAction {
    Resize(u16, u16),
    SleepBeforeRender,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostAction {
    ModeChanged(Mode),
    Open(PathBuf),
    Quit(Option<String>),
    Task(Task),
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

// pub async fn
