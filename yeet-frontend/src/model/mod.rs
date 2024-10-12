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
    pub current_tasks: HashMap<String, CancellationToken>,
    pub files: HashMap<usize, BufferKind>,
    pub history: History,
    pub junk: JunkYard,
    // FIX:
    // seperate windows and buffers with buffer id (file window should be file buffer)
    // connect buffer and window in AppLayout
    // add optional (mvp solution) quake menu like copen
    // add buffer type to differentiate between different window behaviors on keymap messages
    // introduce type for topen (task open)
    // show tasks and update them on start/end
    // introduce dd to cancel task: gray out on dd and remove on task end event
    // add view to app shutdown (open on quite while tasks are running)
    // quit when all tasks finished
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

pub enum BufferKind {
    Directory(usize, Directory),
    Tasks(usize),
    Qfix(usize),
}

pub struct Directory {
    pub current: PathBuffer,
    pub parent: DirectorySibling,
    pub preview: DirectorySibling,
    pub show_border: bool,
}

impl Directory {
    pub fn get_mut_directories(&mut self) -> Vec<(&Path, &mut Buffer)> {
        let parent_content_ref = if let DirectorySibling::Text(path, buffer) = &mut self.parent {
            Some((path.as_path(), buffer))
        } else {
            None
        };

        let preview_content_ref = if let DirectorySibling::Text(path, buffer) = &mut self.preview {
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

impl Default for Directory {
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
pub enum DirectorySibling {
    Image(PathBuf, Box<dyn Protocol>),
    #[default]
    None,
    Text(PathBuf, Buffer),
}

impl DirectorySibling {
    pub fn resolve_path(&self) -> Option<&Path> {
        match self {
            DirectorySibling::Text(path, _) => Some(path),
            DirectorySibling::Image(path, _) => Some(path),
            DirectorySibling::None => None,
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
}
