use std::path::PathBuf;

use ratatui::layout::Rect;
use yeet_keymap::message::Mode;

use crate::layout::{AppLayout, CommandLineLayout};

use self::{
    buffer::{
        viewport::{LineNumber, ViewPort},
        Buffer, Cursor, CursorPosition,
    },
    history::History,
    register::FileRegister,
};

pub mod buffer;
pub mod history;
pub mod register;

#[derive(Debug)]
pub struct Model {
    pub commandline: CommandLine,
    pub current: DirectoryBuffer,
    pub history: History,
    pub key_sequence: String,
    pub layout: AppLayout,
    pub mode: Mode,
    pub mode_before: Option<Mode>,
    pub parent: OptionalDirectoryBuffer,
    pub preview: DirectoryBuffer,
    pub file_register: FileRegister,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            commandline: CommandLine::default(),
            current: DirectoryBuffer {
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
            history: History::default(),
            key_sequence: String::new(),
            layout: AppLayout::new(Rect::default(), 0),
            mode: Mode::default(),
            mode_before: None,
            parent: OptionalDirectoryBuffer {
                buffer: Buffer {
                    cursor: Some(Cursor {
                        horizontial_index: CursorPosition::None,
                        vertical_index: 0,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            preview: DirectoryBuffer::default(),
            file_register: FileRegister::default(),
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
pub struct OptionalDirectoryBuffer {
    pub buffer: Buffer,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct DirectoryBuffer {
    pub buffer: Buffer,
    pub path: PathBuf,
}
