use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use crate::{
    layout::{AppLayout, CommandLineLayout},
    settings::Settings,
};
use ratatui::layout::Rect;
use ratatui_image::protocol::Protocol;
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
    pub do_command: Option<DoCommand>,
    pub files: FileWindow,
    pub history: History,
    pub junk: JunkYard,
    pub key_sequence: String,
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

pub enum DoCommand {
    Cdo(String),
}

pub struct FileWindow {
    pub current: PathBuffer,
    pub parent: BufferType,
    pub preview: BufferType,
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
                    show_border: true,
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
            // parent: DirectoryBuffer {
            //     buffer: Buffer {
            //         cursor: Some(Cursor {
            //             horizontal_index: CursorPosition::None,
            //             vertical_index: 0,
            //             ..Default::default()
            //         }),
            //         show_border: true,
            //         ..Default::default()
            //     },
            //     ..Default::default()
            // },
            preview: Default::default(),
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
    Image(PathBuf, Box<dyn Protocol>),
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
