use std::{
    collections::{HashMap, HashSet},
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
    pub commandline: CommandLine,
    pub contents: Contents,
    pub window: Window,
}

impl Default for App {
    fn default() -> Self {
        let mut buffers = HashMap::new();
        buffers.insert(1, Buffer::Empty);

        Self {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 1,
            },
            window: Window::Directory(
                ViewPort {
                    buffer_id: 1,
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
                    buffer_id: 1,
                    hide_cursor: true,
                    hide_cursor_line: true,
                    ..Default::default()
                },
            ),
        }
    }
}

pub struct Contents {
    pub buffers: HashMap<usize, Buffer>,
    pub latest_buffer_id: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SplitFocus {
    #[default]
    First,
    Second,
}

#[allow(clippy::large_enum_variant)]
pub enum Window {
    Horizontal {
        first: Box<Window>,
        second: Box<Window>,
        focus: SplitFocus,
    },
    Directory(ViewPort, ViewPort, ViewPort),
    Tasks(ViewPort),
}

impl Window {
    pub fn get_height(&self) -> Result<u16, AppError> {
        match self {
            Window::Horizontal { first, second, .. } => {
                Ok(first.get_height()? + second.get_height()?)
            }
            Window::Directory(_, vp, _) => Ok(vp.height),
            Window::Tasks(vp) => Ok(vp.height),
        }
    }

    pub fn focused_viewport(&self) -> &ViewPort {
        match self {
            Window::Horizontal {
                first,
                second,
                focus,
            } => match focus {
                SplitFocus::First => first.focused_viewport(),
                SplitFocus::Second => second.focused_viewport(),
            },
            Window::Directory(_, vp, _) => vp,
            Window::Tasks(vp) => vp,
        }
    }

    pub fn focused_viewport_mut(&mut self) -> &mut ViewPort {
        match self {
            Window::Horizontal {
                first,
                second,
                focus,
            } => match focus {
                SplitFocus::First => first.focused_viewport_mut(),
                SplitFocus::Second => second.focused_viewport_mut(),
            },
            Window::Directory(_, vp, _) => vp,
            Window::Tasks(vp) => vp,
        }
    }

    pub fn buffer_ids(&self) -> HashSet<usize> {
        match self {
            Window::Horizontal { first, second, .. } => {
                let mut ids = first.buffer_ids();
                ids.extend(second.buffer_ids());
                ids
            }
            Window::Directory(parent, current, preview) => {
                HashSet::from([parent.buffer_id, current.buffer_id, preview.buffer_id])
            }
            Window::Tasks(vp) => HashSet::from([vp.buffer_id]),
        }
    }

    /// Returns the total rendered height including per-leaf statusline rows.
    pub fn get_rendered_height(&self) -> Result<u16, AppError> {
        match self {
            Window::Horizontal { first, second, .. } => {
                Ok(first.get_rendered_height()? + second.get_rendered_height()?)
            }
            Window::Directory(_, vp, _) => Ok(vp.height + 1),
            Window::Tasks(vp) => Ok(vp.height + 1),
        }
    }

    pub fn contains_tasks(&self) -> bool {
        match self {
            Window::Horizontal { first, second, .. } => {
                first.contains_tasks() || second.contains_tasks()
            }
            Window::Directory(_, _, _) => false,
            Window::Tasks(_) => true,
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Window::Directory(Default::default(), Default::default(), Default::default())
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
    Tasks(TasksBuffer),
    Empty,
}

impl Buffer {
    pub fn resolve_path(&self) -> Option<&Path> {
        match self {
            Buffer::Directory(it) => it.resolve_path(),
            Buffer::Content(it) => it.resolve_path(),
            Buffer::Image(it) => it.resolve_path(),
            Buffer::PathReference(path) => {
                if path.as_os_str().is_empty() {
                    None
                } else {
                    Some(path.as_path())
                }
            }
            Buffer::Tasks(_) | Buffer::Empty => None,
        }
    }
}

#[derive(Default)]
pub struct TasksBuffer {
    pub buffer: TextBuffer,
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

#[cfg(test)]
mod test {
    use yeet_buffer::model::viewport::ViewPort;

    use super::*;

    #[test]
    fn split_focus_default_is_first() {
        assert_eq!(SplitFocus::default(), SplitFocus::First);
    }

    #[test]
    fn window_tasks_construction_and_pattern_match() {
        let task_window = Window::Tasks(ViewPort::default());
        assert!(matches!(task_window, Window::Tasks(_)));
    }

    #[test]
    fn window_horizontal_struct_variant_construction() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::First,
        };
        assert!(matches!(tree, Window::Horizontal { .. }));
    }

    #[test]
    fn buffer_tasks_construction_and_pattern_match() {
        let buf = Buffer::Tasks(TasksBuffer {
            buffer: TextBuffer::default(),
        });
        assert!(matches!(buf, Buffer::Tasks(_)));
    }

    #[test]
    fn get_height_horizontal_sums_children() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Tasks(ViewPort {
                height: 10,
                ..Default::default()
            })),
            second: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    height: 15,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            focus: SplitFocus::First,
        };
        assert_eq!(tree.get_height().unwrap(), 25);
    }

    #[test]
    fn get_height_tasks_returns_viewport_height() {
        let w = Window::Tasks(ViewPort {
            height: 7,
            ..Default::default()
        });
        assert_eq!(w.get_height().unwrap(), 7);
    }

    #[test]
    fn focused_viewport_follows_split_focus_first() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    height: 42,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort {
                height: 10,
                ..Default::default()
            })),
            focus: SplitFocus::First,
        };
        assert_eq!(tree.focused_viewport().height, 42);
    }

    #[test]
    fn focused_viewport_follows_split_focus_second() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    height: 42,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort {
                height: 10,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };
        assert_eq!(tree.focused_viewport().height, 10);
    }

    #[test]
    fn buffer_ids_collects_all_from_nested_tree() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort {
                    buffer_id: 1,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: 2,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: 3,
                    ..Default::default()
                },
            )),
            second: Box::new(Window::Tasks(ViewPort {
                buffer_id: 4,
                ..Default::default()
            })),
            focus: SplitFocus::First,
        };
        let ids = tree.buffer_ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
        assert!(ids.contains(&4));
    }
}
