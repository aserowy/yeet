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
    Cursor, Mode, TextBuffer,
};

use self::{history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, register::Register};

pub mod history;
pub mod junkyard;
pub mod mark;
pub mod qfix;
pub mod register;

pub struct Model {
    pub commandline: CommandLine,
    pub current_tasks: HashMap<String, CurrentTask>,
    pub buffer: Buffer,
    pub history: History,
    pub junk: JunkYard,
    pub latest_task_id: u16,
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

impl Default for Model {
    fn default() -> Self {
        Self {
            commandline: Default::default(),
            current_tasks: Default::default(),
            buffer: Buffer::FileTree(Default::default()),
            history: Default::default(),
            junk: Default::default(),
            latest_task_id: Default::default(),
            layout: Default::default(),
            marks: Default::default(),
            mode: Default::default(),
            mode_before: Default::default(),
            qfix: Default::default(),
            register: Default::default(),
            remaining_keysequence: Default::default(),
            settings: Default::default(),
            watches: Default::default(),
        }
    }
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("current_tasks", &self.current_tasks)
            .field("junk", &self.junk)
            .field("marks", &self.marks)
            .field("qfix", &self.qfix)
            .field("settings", &self.settings)
            .finish()
    }
}

#[derive(Debug)]
pub struct CurrentTask {
    pub external_id: String,
    pub id: u16,
    pub token: CancellationToken,
}

pub struct CommandLine {
    pub buffer: TextBuffer,
    pub cursor: Option<Cursor>,
    pub key_sequence: String,
    pub layout: CommandLineLayout,
    pub viewport: ViewPort,
}

impl Default for CommandLine {
    fn default() -> Self {
        Self {
            cursor: Some(Cursor {
                hide_cursor: true,
                hide_cursor_line: true,
                vertical_index: 0,
                ..Default::default()
            }),
            buffer: Default::default(),
            key_sequence: "".to_owned(),
            layout: CommandLineLayout::new(Rect::default(), 0),
            viewport: Default::default(),
        }
    }
}

pub enum Buffer {
    FileTree(FileTreeBuffer),
    Text(TextBuffer),
}

pub struct FileTreeBuffer {
    pub current: DirectoryBuffer,
    pub current_vp: ViewPort,
    pub current_cursor: Option<Cursor>,
    pub parent: FileTreeBufferSectionBuffer,
    pub parent_vp: ViewPort,
    pub parent_cursor: Option<Cursor>,
    pub preview: FileTreeBufferSectionBuffer,
    pub preview_vp: ViewPort,
    pub preview_cursor: Option<Cursor>,
    pub show_border: bool,
}

impl FileTreeBuffer {
    pub fn get_mut_directories(
        &mut self,
    ) -> Vec<(&Path, &mut ViewPort, &mut Option<Cursor>, &mut TextBuffer)> {
        let parent_content_ref =
            if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut self.parent {
                Some((
                    path.as_path(),
                    &mut self.parent_vp,
                    &mut self.parent_cursor,
                    buffer,
                ))
            } else {
                None
            };

        let preview_content_ref =
            if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut self.preview {
                Some((
                    path.as_path(),
                    &mut self.preview_vp,
                    &mut self.preview_cursor,
                    buffer,
                ))
            } else {
                None
            };

        vec![
            Some((
                self.current.path.as_path(),
                &mut self.current_vp,
                &mut self.current_cursor,
                &mut self.current.buffer,
            )),
            parent_content_ref,
            preview_content_ref,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }
}

impl Default for FileTreeBuffer {
    fn default() -> Self {
        Self {
            current: Default::default(),
            current_cursor: Some(Cursor::default()),
            current_vp: ViewPort {
                line_number: LineNumber::Relative,
                line_number_width: 3,
                ..Default::default()
            },
            parent: Default::default(),
            parent_vp: Default::default(),
            parent_cursor: Default::default(),
            preview: Default::default(),
            preview_vp: Default::default(),
            preview_cursor: Default::default(),
            show_border: true,
        }
    }
}

#[derive(Debug)]
pub enum FileTreeBufferSection {
    Current,
    Parent,
    Preview,
}

// NOTE: most of the time Text is used. Thus, boxing Buffer would only increase hassle to work
// with this BufferType.
#[allow(clippy::large_enum_variant)]
#[derive(Default)]
pub enum FileTreeBufferSectionBuffer {
    Image(PathBuf, Protocol),
    #[default]
    None,
    Text(PathBuf, TextBuffer),
}

impl FileTreeBufferSectionBuffer {
    pub fn resolve_path(&self) -> Option<&Path> {
        match self {
            FileTreeBufferSectionBuffer::Text(path, _) => Some(path),
            FileTreeBufferSectionBuffer::Image(path, _) => Some(path),
            FileTreeBufferSectionBuffer::None => None,
        }
    }
}

#[derive(Default)]
pub struct DirectoryBuffer {
    pub buffer: TextBuffer,
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
