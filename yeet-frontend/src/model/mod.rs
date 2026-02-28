use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{error::AppError, settings::Settings};
use ratatui_image::protocol::Protocol;
use tokio_util::sync::CancellationToken;
use yeet_buffer::model::{
    viewport::{LineNumber, ViewPort},
    Cursor, Mode, TextBuffer,
};

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
                    hide_cursor: true,
                    show_border: true,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: 1,
                    line_number: LineNumber::Relative,
                    line_number_width: 3,
                    show_border: true,
                    sign_column_width: 2,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: 3,
                    hide_cursor: true,
                    hide_cursor_line: true,
                    ..Default::default()
                },
            ),
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
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
    pub pending_path_events: Vec<PendingPathEvent>,
    pub tasks: Tasks,
    pub watches: Vec<PathBuf>,
}

pub enum PendingPathEvent {
    Added(Vec<PathBuf>),
    Removed(PathBuf),
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
                ..Default::default()
            },
            key_sequence: "".to_owned(),
            viewport: yeet_buffer::model::viewport::ViewPort {
                hide_cursor: true,
                hide_cursor_line: true,
                ..Default::default()
            },
        }
    }
}

pub enum Buffer {
    Directory(DirectoryBuffer),
    Image(PreviewImageBuffer),
    Content(ContentBuffer),
    PathReference(PathBuf),
    Empty,
}

#[derive(Default)]
pub struct ContentBuffer {
    pub path: PathBuf,
    pub buffer: TextBuffer,
}

impl ContentBuffer {
    pub fn resolve_path(&self) -> Option<&Path> {
        if self.path.as_os_str().is_empty() {
            None
        } else {
            Some(self.path.as_path())
        }
    }
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
    PartiallyLoaded,
    Ready,
    #[default]
    Uninitialized,
}

pub fn get_selected_path(buffer: &DirectoryBuffer, cursor: &Cursor) -> Option<PathBuf> {
    get_selected_path_with_base(&buffer.path, &buffer.buffer, cursor, |path| path.exists())
}

pub fn get_selected_path_with_base(
    base_path: &Path,
    text_buffer: &TextBuffer,
    cursor: &Cursor,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if text_buffer.lines.is_empty() {
        return None;
    }

    let current = &text_buffer.lines.get(cursor.vertical_index)?;
    if current.content.is_empty() {
        return None;
    }

    let target = base_path.join(current.content.to_stripped_string());

    if exists(&target) {
        Some(target)
    } else {
        None
    }
}
