use std::{env, path::PathBuf};

use yate_keymap::message::Mode;

use self::{
    buffer::{
        viewport::{LineNumber, ViewPort},
        Buffer, Cursor, CursorPosition,
    },
    history::History,
};

pub mod buffer;
pub mod history;

#[derive(Debug)]
pub struct Model {
    pub commandline: Buffer,
    pub current_directory: Buffer,
    pub current_path: PathBuf,
    pub history: History,
    pub key_sequence: String,
    pub mode: Mode,
    pub parent_directory: Buffer,
    pub preview: Buffer,
}

impl Default for Model {
    fn default() -> Self {
        let current_path = get_current_path();

        Self {
            commandline: Buffer {
                cursor: Some(Cursor {
                    hide_cursor: true,
                    hide_cursor_line: true,
                    vertical_index: 0,
                    ..Default::default()
                }),
                ..Default::default()
            },
            current_path,
            current_directory: Buffer {
                cursor: Some(Cursor::default()),
                view_port: ViewPort {
                    line_number: LineNumber::Relative,
                    line_number_width: 3,
                    ..Default::default()
                },
                ..Default::default()
            },
            history: History::default(),
            key_sequence: String::new(),
            mode: Mode::default(),
            parent_directory: Buffer {
                cursor: Some(Cursor {
                    horizontial_index: CursorPosition::None,
                    vertical_index: 0,
                    ..Default::default()
                }),
                ..Default::default()
            },
            preview: Buffer::default(),
        }
    }
}

fn get_current_path() -> PathBuf {
    if let Ok(path) = env::current_dir() {
        path
    } else if let Some(val) = dirs::home_dir() {
        val
    } else {
        // TODO: log error
        PathBuf::new()
    }
}
