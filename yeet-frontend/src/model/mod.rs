use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    layout::{AppLayout, CommandLineLayout},
    settings::Settings,
};
use ratatui::layout::Rect;
use ratatui_image::protocol::Protocol;
use tokio_util::sync::CancellationToken;
use yeet_buffer::model::{
    viewport::{LineNumber, ViewPort},
    Buffer, Cursor, Mode,
};

use self::{history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, register::Register};

pub mod history;
pub mod junkyard;
pub mod mark;
pub mod qfix;
pub mod register;

#[derive(Default)]
pub struct Model {
    pub commandline: CommandLine,
    pub current_tasks: HashMap<String, CurrentTask>,
    // pub current_tasks: HashMap<String, CancellationToken>,
    pub files: FileWindow,
    pub history: History,
    pub junk: JunkYard,
    pub layout: AppLayout,
    pub marks: Marks,
    pub mode: Mode,
    pub mode_before: Option<Mode>,
    pub qfix: QuickFix,
    pub register: Register,
    pub remaining_keysequence: Option<String>,
    pub settings: Settings,
    pub watches: Vec<PathBuf>,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("junk", &self.junk)
            .field("marks", &self.marks)
            .field("qfix", &self.qfix)
            .field("settings", &self.settings)
            .finish()
    }
}

#[derive(Debug)]
pub struct CurrentTask {
    pub token: CancellationToken,
    pub id: usize,
    pub description: String,
}

pub struct FileWindow {
    pub current: PathBuffer,
    pub parent: BufferType,
    pub preview: BufferType,
    pub show_border: bool,
}

impl FileWindow {
    pub fn get_mut_directories(&mut self) -> Vec<(&Path, &mut Buffer)> {
        let parent_content_ref = if let BufferType::Text(path, buffer) = &mut self.parent {
            Some((path.as_path(), buffer))
        } else {
            None
        };

        let preview_content_ref = if let BufferType::Text(path, buffer) = &mut self.preview {
            Some((path.as_path(), buffer))
        } else {
            None
        };

        vec![
            Some((self.current.path.as_path(), &mut self.current.buffer)),
            parent_content_ref,
            preview_content_ref,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }
}

impl Default for FileWindow {
    fn default() -> Self {
        Self {
            current: PathBuffer {
                buffer: Buffer {
                    cursor: Some(Cursor::default()),
                    view_port: ViewPort {
                        line_number: LineNumber::Relative,
                        line_number_width: 3,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            parent: Default::default(),
            preview: Default::default(),
            show_border: true,
        }
    }
}

#[derive(Debug)]
pub enum WindowType {
    Current,
    Parent,
    Preview,
}

#[derive(Default)]
pub enum BufferType {
    Image(PathBuf, Protocol),
    #[default]
    None,
    Text(PathBuf, Buffer),
}

impl BufferType {
    pub fn resolve_path(&self) -> Option<&Path> {
        match self {
            BufferType::Text(path, _) => Some(path),
            BufferType::Image(path, _) => Some(path),
            BufferType::None => None,
        }
    }
}

pub struct CommandLine {
    pub buffer: Buffer,
    pub key_sequence: String,
    pub layout: CommandLineLayout,
}

impl Default for CommandLine {
    fn default() -> Self {
        Self {
            buffer: Buffer {
                cursor: Some(Cursor {
                    hide_cursor: true,
                    hide_cursor_line: true,
                    vertical_index: 0,
                    ..Default::default()
                }),
                ..Default::default()
            },
            key_sequence: "".to_owned(),
            layout: CommandLineLayout::new(Rect::default(), 0),
        }
    }
}

#[derive(Default)]
pub struct PathBuffer {
    pub buffer: Buffer,
    pub path: PathBuf,
    pub state: DirectoryBufferState,
}

#[derive(Debug, Default, PartialEq)]
pub enum DirectoryBufferState {
    Loading,
    PartiallyLoaded,
    Ready,
    #[default]
    Uninitialized,
}
