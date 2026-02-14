use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{error::AppError, settings::Settings};
use ratatui_image::protocol::Protocol;
use tokio_util::sync::CancellationToken;
use yeet_buffer::model::{viewport::ViewPort, Mode, TextBuffer};

use self::{history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, register::Register};

pub mod history;
pub mod junkyard;
pub mod mark;
pub mod qfix;
pub mod register;

#[derive(Default)]
pub struct Model {
    pub app: App,
    pub settings: Settings,
    pub state: State,
}

pub struct App {
    pub buffers: HashMap<usize, Buffer>,
    pub commandline: CommandLine,
    pub latest_buffer_id: usize,
    pub window: Window,
}

impl Default for App {
    fn default() -> Self {
        let mut buffers = HashMap::new();
        buffers.insert(1, Buffer::Directory(Default::default()));
        buffers.insert(2, Buffer::Directory(Default::default()));
        buffers.insert(3, Buffer::Directory(Default::default()));

        Self {
            buffers,
            commandline: Default::default(),
            latest_buffer_id: 3,
            window: Window::Directory(
                ViewPort {
                    buffer_id: 2,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: 1,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: 3,
                    ..Default::default()
                },
            ),
        }
    }
}

#[allow(dead_code)]
pub enum Window {
    Horizontal(Box<Window>, Box<Window>),
    Directory(ViewPort, ViewPort, ViewPort),
}

impl Window {
    pub fn get_height(&self) -> Result<u16, AppError> {
        match self {
            Window::Horizontal(_, _) => todo!(),
            Window::Directory(_, vp, _) => Ok(vp.height),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub history: History,
    pub junk: JunkYard,
    pub marks: Marks,
    pub modes: ModeState,
    pub qfix: QuickFix,
    pub register: Register,
    pub remaining_keysequence: Option<String>,
    pub tasks: Tasks,
    pub watches: Vec<PathBuf>,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("settings", &self.settings)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct ModeState {
    pub current: Mode,
    pub previous: Option<Mode>,
}

#[derive(Debug, Default)]
pub struct Tasks {
    pub latest_id: u16,
    pub running: HashMap<String, CurrentTask>,
}

#[derive(Debug)]
pub struct CurrentTask {
    pub external_id: String,
    pub id: u16,
    pub token: CancellationToken,
}

pub struct CommandLine {
    pub buffer: TextBuffer,
    pub key_sequence: String,
    pub viewport: ViewPort,
}

impl Default for CommandLine {
    fn default() -> Self {
        Self {
            buffer: TextBuffer {
                cursor: yeet_buffer::model::Cursor {
                    hide_cursor: true,
                    hide_cursor_line: true,
                    vertical_index: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
            key_sequence: "".to_owned(),
            viewport: Default::default(),
        }
    }
}

pub enum Buffer {
    Directory(DirectoryBuffer),
    PreviewImage(PreviewImageBuffer),
    _Text(Box<TextBuffer>),
}

#[derive(Debug)]
pub enum DirectoryPane {
    Current,
    Parent,
    Preview,
}

pub struct PreviewImageBuffer {
    pub path: PathBuf,
    pub protocol: Protocol,
}

impl PreviewImageBuffer {
    pub fn resolve_path(&self) -> Option<&Path> {
        if self.path.as_os_str().is_empty() {
            None
        } else {
            Some(self.path.as_path())
        }
    }
}

#[derive(Default)]
pub struct DirectoryBuffer {
    pub buffer: TextBuffer,
    pub path: PathBuf,
    pub state: DirectoryBufferState,
}

impl DirectoryBuffer {
    pub fn resolve_path(&self) -> Option<&Path> {
        if self.path.as_os_str().is_empty() {
            None
        } else {
            Some(self.path.as_path())
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum DirectoryBufferState {
    Loading,
    PartiallyLoaded,
    Ready,
    #[default]
    Uninitialized,
}
