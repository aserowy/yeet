use std::path::{Path, PathBuf};

use ratatui::layout::Rect;
use yeet_buffer::model::{
    viewport::{LineNumber, ViewPort},
    Buffer, Cursor, CursorPosition, Mode, SearchModel,
};

use crate::{
    layout::{AppLayout, CommandLineLayout},
    settings::Settings,
};

use self::{history::History, mark::Marks, qfix::QuickFix, register::JunkYard};

pub mod history;
pub mod mark;
pub mod qfix;
pub mod register;

#[derive(Debug, Default)]
pub struct Model {
    pub commandline: CommandLine,
    pub file_buffer: FileBuffer,
    pub history: History,
    pub junk: JunkYard,
    pub key_sequence: String,
    pub layout: AppLayout,
    pub marks: Marks,
    pub mode: Mode,
    pub mode_before: Option<Mode>,
    pub qfix: QuickFix,
    pub search: Option<SearchModel>,
    pub settings: Settings,
    pub watches: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct FileBuffer {
    pub current: DirectoryBuffer<PathBuf>,
    pub parent: DirectoryBuffer<Option<PathBuf>>,
    pub preview: DirectoryBuffer<Option<PathBuf>>,
}

impl FileBuffer {
    pub fn get_mut_directories(&mut self) -> Vec<(&Path, &mut DirectoryBufferState, &mut Buffer)> {
        vec![
            self.current.as_content_ref(),
            self.parent.as_content_ref(),
            self.preview.as_content_ref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }
}

impl Default for FileBuffer {
    fn default() -> Self {
        Self {
            current: DirectoryBuffer {
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
            parent: DirectoryBuffer::<Option<PathBuf>> {
                buffer: Buffer {
                    cursor: Some(Cursor {
                        horizontal_index: CursorPosition::None,
                        vertical_index: 0,
                        ..Default::default()
                    }),
                    show_border: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            preview: DirectoryBuffer::<Option<PathBuf>>::default(),
        }
    }
}

#[derive(Debug)]
pub struct CommandLine {
    pub buffer: Buffer,
    pub layout: CommandLineLayout,
    pub state: CommandLineState,
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
            state: CommandLineState::default(),
        }
    }
}

#[derive(Debug, Default)]
pub enum CommandLineState {
    #[default]
    Default,
    WaitingForInput,
}

#[derive(Debug, Default)]
pub struct DirectoryBuffer<T> {
    pub buffer: Buffer,
    pub path: T,
    pub state: DirectoryBufferState,
}

pub type DirectoryContentRef<'a> = (&'a Path, &'a mut DirectoryBufferState, &'a mut Buffer);

impl DirectoryBuffer<PathBuf> {
    pub fn as_content_ref(&mut self) -> Option<DirectoryContentRef> {
        Some((&self.path, &mut self.state, &mut self.buffer))
    }
}

impl DirectoryBuffer<Option<PathBuf>> {
    pub fn as_content_ref(&mut self) -> Option<DirectoryContentRef> {
        match &self.path {
            Some(path) => Some((path, &mut self.state, &mut self.buffer)),
            None => None,
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
