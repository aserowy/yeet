use std::path::PathBuf;

use yeet_keymap::message::Mode;

use self::{
    buffer::{
        viewport::{LineNumber, ViewPort},
        Buffer, Cursor, CursorPosition,
    },
    history::History,
    register::Register,
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
    pub mode: Mode,
    pub mode_before: Option<Mode>,
    pub parent: OptionalDirectoryBuffer,
    pub preview: DirectoryBuffer,
    pub register: Register,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            commandline: CommandLine {
                buffer: Buffer {
                    cursor: Some(Cursor {
                        hide_cursor: true,
                        hide_cursor_line: true,
                        vertical_index: 0,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
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
            register: Register::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandLine {
    pub buffer: Buffer,
    pub state: CommandLineState,
}

#[derive(Debug, Default)]
pub enum CommandLineState {
    #[default]
    Default,

    _WaitingForInput,
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
