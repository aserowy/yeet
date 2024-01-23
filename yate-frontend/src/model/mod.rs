use std::{env, path::PathBuf};

use yate_keymap::message::Mode;

use self::buffer::{Buffer, Cursor, CursorPosition, LineNumber};

pub mod buffer;

#[derive(Debug)]
pub struct Model {
    pub current_directory: Buffer,
    pub current_path: PathBuf,
    pub key_sequence: String,
    pub mode: Mode,
    pub parent_directory: Buffer,
    pub preview: Buffer,
}

impl Default for Model {
    fn default() -> Self {
        let current_path = get_current_path();

        Self {
            current_path,
            current_directory: Buffer {
                cursor: Some(Cursor::default()),
                lines: Default::default(),
                view_port: buffer::ViewPort {
                    line_number: LineNumber::Relative,
                    line_number_width: 3,
                    ..Default::default()
                },
            },
            key_sequence: String::new(),
            mode: Mode::default(),
            parent_directory: Buffer {
                cursor: Some(Cursor {
                    horizontial_index: CursorPosition::None,
                    vertical_index: 0,
                }),
                lines: Default::default(),
                view_port: Default::default(),
            },
            preview: Buffer::default(),
        }
    }
}

fn get_current_path() -> PathBuf {
    if let Ok(path) = env::current_dir() {
        return path;
    }

    dirs::home_dir().unwrap()
}
