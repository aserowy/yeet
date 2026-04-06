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
use yeet_lua::LuaConfiguration;

use self::{history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, register::Register};

pub mod history;
pub mod junkyard;
pub mod mark;
pub mod qfix;
pub mod register;

#[derive(Default)]
pub struct Model {
    pub app: App,
    pub lua: Option<LuaConfiguration>,
    pub settings: Settings,
    pub state: State,
}

pub struct App {
    pub commandline: CommandLine,
    pub contents: Contents,
    pub tabs: HashMap<usize, Window>,
    pub current_tab_id: usize,
}

impl Default for App {
    fn default() -> Self {
        let mut buffers = HashMap::new();
        buffers.insert(1, Buffer::Empty);

        let window = Window::create(1, 1, 1);
        let mut tabs = HashMap::new();
        tabs.insert(1, window);

        Self {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 1,
            },
            tabs,
            current_tab_id: 1,
        }
    }
}

impl App {
    pub fn current_window(&self) -> Result<&Window, AppError> {
        match self.tabs.get(&self.current_tab_id) {
            Some(window) => Ok(window),
            None => {
                let err = AppError::TabNotFound(self.current_tab_id);
                tracing::error!("Failed to resolve current window: {}", err);
                Err(err)
            }
        }
    }

    pub fn current_window_mut(&mut self) -> Result<&mut Window, AppError> {
        match self.tabs.get_mut(&self.current_tab_id) {
            Some(window) => Ok(window),
            None => {
                let err = AppError::TabNotFound(self.current_tab_id);
                tracing::error!("Failed to resolve current window: {}", err);
                Err(err)
            }
        }
    }

    pub fn current_window_and_contents_mut(
        &mut self,
    ) -> Result<(&mut Window, &mut Contents), AppError> {
        let App {
            contents,
            tabs,
            current_tab_id,
            ..
        } = self;
        let window = match tabs.get_mut(current_tab_id) {
            Some(window) => window,
            None => {
                let err = AppError::TabNotFound(*current_tab_id);
                tracing::error!("Failed to resolve current window: {}", err);
                return Err(err);
            }
        };
        Ok((window, contents))
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
    Vertical {
        first: Box<Window>,
        second: Box<Window>,
        focus: SplitFocus,
    },
    Directory(ViewPort, ViewPort, ViewPort),
    Help(ViewPort),
    QuickFix(ViewPort),
    Tasks(ViewPort),
}

impl Window {
    pub fn create(parent_id: usize, current_id: usize, preview_id: usize) -> Window {
        Window::Directory(
            ViewPort {
                buffer_id: parent_id,
                hide_cursor: true,
                show_border: true,
                ..Default::default()
            },
            ViewPort {
                buffer_id: current_id,
                line_number: LineNumber::Relative,
                line_number_width: 3,
                show_border: true,
                sign_column_width: 2,
                ..Default::default()
            },
            ViewPort {
                buffer_id: preview_id,
                hide_cursor: true,
                hide_cursor_line: true,
                ..Default::default()
            },
        )
    }

    pub fn focused_viewport(&self) -> &ViewPort {
        match self {
            Window::Horizontal {
                first,
                second,
                focus,
            }
            | Window::Vertical {
                first,
                second,
                focus,
            } => match focus {
                SplitFocus::First => first.focused_viewport(),
                SplitFocus::Second => second.focused_viewport(),
            },
            Window::Directory(_, vp, _)
            | Window::Help(vp)
            | Window::QuickFix(vp)
            | Window::Tasks(vp) => vp,
        }
    }

    pub fn focused_window_mut(&mut self) -> &mut Window {
        match self {
            Window::Horizontal {
                first,
                second,
                focus,
            }
            | Window::Vertical {
                first,
                second,
                focus,
            } => match focus {
                SplitFocus::First => first.focused_window_mut(),
                SplitFocus::Second => second.focused_window_mut(),
            },
            Window::Directory(..) | Window::Help(_) | Window::QuickFix(_) | Window::Tasks(_) => {
                self
            }
        }
    }

    pub fn focused_viewport_mut(&mut self) -> &mut ViewPort {
        match self.focused_window_mut() {
            Window::Directory(_, vp, _)
            | Window::Help(vp)
            | Window::QuickFix(vp)
            | Window::Tasks(vp) => vp,
            Window::Horizontal { .. } | Window::Vertical { .. } => {
                unreachable!("focused_window_mut should have returned a non-split window")
            }
        }
    }

    pub fn buffer_ids(&self) -> HashSet<usize> {
        match self {
            Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
                let mut ids = first.buffer_ids();
                ids.extend(second.buffer_ids());
                ids
            }
            Window::Directory(parent, current, preview) => {
                HashSet::from([parent.buffer_id, current.buffer_id, preview.buffer_id])
            }
            Window::Help(vp) | Window::QuickFix(vp) | Window::Tasks(vp) => {
                HashSet::from([vp.buffer_id])
            }
        }
    }

    pub fn contains_tasks(&self) -> bool {
        match self {
            Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
                first.contains_tasks() || second.contains_tasks()
            }
            Window::Directory(_, _, _) | Window::Help(_) | Window::QuickFix(_) => false,
            Window::Tasks(_) => true,
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn close_focused(self) -> Result<(Window, Window), Window> {
        match self {
            Window::Horizontal {
                first,
                second,
                focus,
            } => Self::close_focused_in_split(*first, *second, focus, |f, s, focus| {
                Window::Horizontal {
                    first: Box::new(f),
                    second: Box::new(s),
                    focus,
                }
            }),
            Window::Vertical {
                first,
                second,
                focus,
            } => Self::close_focused_in_split(*first, *second, focus, |f, s, focus| {
                Window::Vertical {
                    first: Box::new(f),
                    second: Box::new(s),
                    focus,
                }
            }),
            leaf => Err(leaf),
        }
    }

    #[allow(clippy::result_large_err)]
    fn close_focused_in_split(
        first: Window,
        second: Window,
        focus: SplitFocus,
        rebuild: impl FnOnce(Window, Window, SplitFocus) -> Window,
    ) -> Result<(Window, Window), Window> {
        let (focused, sibling, focused_is_first) = match focus {
            SplitFocus::First => (first, second, true),
            SplitFocus::Second => (second, first, false),
        };

        match focused.close_focused() {
            Ok((kept, dropped)) => {
                let (new_first, new_second) = if focused_is_first {
                    (kept, sibling)
                } else {
                    (sibling, kept)
                };
                Ok((rebuild(new_first, new_second, focus), dropped))
            }
            Err(leaf) => Ok((sibling, leaf)),
        }
    }

    pub fn set_wrap(&mut self, wrap: bool) {
        match self.focused_window_mut() {
            Window::Directory(parent, current, preview) => {
                parent.wrap = wrap;
                current.wrap = wrap;
                preview.wrap = wrap;
            }
            Window::Help(vp) | Window::QuickFix(vp) | Window::Tasks(vp) => {
                vp.wrap = wrap;
            }
            Window::Horizontal { .. } | Window::Vertical { .. } => {
                unreachable!("focused_window_mut should have returned a non-split window")
            }
        }
    }

    pub fn contains_quickfix(&self) -> bool {
        match self {
            Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
                first.contains_quickfix() || second.contains_quickfix()
            }
            Window::Directory(_, _, _) | Window::Help(_) | Window::Tasks(_) => false,
            Window::QuickFix(_) => true,
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
            buffer: TextBuffer::default(),
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
    Help(HelpBuffer),
    PathReference(PathBuf),
    QuickFix(QuickFixBuffer),
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
            Buffer::Help(_) | Buffer::QuickFix(_) | Buffer::Tasks(_) | Buffer::Empty => None,
        }
    }
}

#[derive(Default)]
pub struct HelpBuffer {
    pub buffer: TextBuffer,
}

#[derive(Default)]
pub struct QuickFixBuffer {
    pub buffer: TextBuffer,
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
    fn app_default_sets_current_tab() {
        let app = App::default();
        assert_eq!(app.current_tab_id, 1);
        assert_eq!(app.tabs.len(), 1);
        assert!(matches!(app.tabs.get(&1), Some(Window::Directory(_, _, _))));
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
    fn window_vertical_construction_and_pattern_match() {
        let tree = Window::Vertical {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            focus: SplitFocus::First,
        };
        assert!(matches!(tree, Window::Vertical { .. }));
    }

    #[test]
    fn buffer_tasks_construction_and_pattern_match() {
        let buf = Buffer::Tasks(TasksBuffer {
            buffer: TextBuffer::default(),
        });
        assert!(matches!(buf, Buffer::Tasks(_)));
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

    #[test]
    fn focused_viewport_follows_vertical_split_focus() {
        let tree = Window::Vertical {
            first: Box::new(Window::Tasks(ViewPort {
                height: 10,
                ..Default::default()
            })),
            second: Box::new(Window::Tasks(ViewPort {
                height: 20,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };
        assert_eq!(tree.focused_viewport().height, 20);
    }

    #[test]
    fn buffer_ids_collects_from_vertical() {
        let tree = Window::Vertical {
            first: Box::new(Window::Tasks(ViewPort {
                buffer_id: 1,
                ..Default::default()
            })),
            second: Box::new(Window::Tasks(ViewPort {
                buffer_id: 2,
                ..Default::default()
            })),
            focus: SplitFocus::First,
        };
        let ids = tree.buffer_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn contains_tasks_in_vertical() {
        let tree = Window::Vertical {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::First,
        };
        assert!(tree.contains_tasks());
    }

    #[test]
    fn contains_tasks_false_in_vertical_without_tasks() {
        let tree = Window::Vertical {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            focus: SplitFocus::First,
        };
        assert!(!tree.contains_tasks());
    }

    #[test]
    fn window_quickfix_construction_and_pattern_match() {
        let qf_window = Window::QuickFix(ViewPort::default());
        assert!(matches!(qf_window, Window::QuickFix(_)));
    }

    #[test]
    fn buffer_quickfix_construction_and_pattern_match() {
        let buf = Buffer::QuickFix(QuickFixBuffer {
            buffer: TextBuffer::default(),
        });
        assert!(matches!(buf, Buffer::QuickFix(_)));
    }

    #[test]
    fn contains_quickfix_true_in_horizontal() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::QuickFix(ViewPort::default())),
            focus: SplitFocus::First,
        };
        assert!(tree.contains_quickfix());
        assert!(!tree.contains_tasks());
    }

    #[test]
    fn contains_quickfix_false_without_quickfix() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::First,
        };
        assert!(!tree.contains_quickfix());
    }

    #[test]
    fn focused_viewport_follows_quickfix_in_split() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    height: 42,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            second: Box::new(Window::QuickFix(ViewPort {
                height: 10,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };
        assert_eq!(tree.focused_viewport().height, 10);
    }

    #[test]
    fn buffer_ids_collects_quickfix() {
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
            second: Box::new(Window::QuickFix(ViewPort {
                buffer_id: 4,
                ..Default::default()
            })),
            focus: SplitFocus::First,
        };
        let ids = tree.buffer_ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&4));
    }

    #[test]
    fn close_focused_horizontal_first_child() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Tasks(ViewPort {
                buffer_id: 1,
                ..Default::default()
            })),
            second: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    buffer_id: 2,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            focus: SplitFocus::First,
        };
        let (kept, dropped) = tree.close_focused().ok().unwrap();
        assert!(matches!(kept, Window::Directory(..)));
        assert!(matches!(dropped, Window::Tasks(_)));
        assert!(dropped.buffer_ids().contains(&1));
    }

    #[test]
    fn close_focused_horizontal_second_child() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    buffer_id: 2,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort {
                buffer_id: 1,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };
        let (kept, dropped) = tree.close_focused().ok().unwrap();
        assert!(matches!(kept, Window::Directory(..)));
        assert!(matches!(dropped, Window::Tasks(_)));
    }

    #[test]
    fn close_focused_vertical_second_child() {
        let tree = Window::Vertical {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
        };
        let (kept, dropped) = tree.close_focused().ok().unwrap();
        assert!(matches!(kept, Window::Directory(..)));
        assert!(matches!(dropped, Window::Tasks(_)));
    }

    #[test]
    fn close_focused_nested_split_closes_innermost() {
        let tree = Window::Horizontal {
            first: Box::new(Window::Vertical {
                first: Box::new(Window::Directory(
                    ViewPort::default(),
                    ViewPort {
                        buffer_id: 10,
                        ..Default::default()
                    },
                    ViewPort::default(),
                )),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 20,
                    ..Default::default()
                })),
                focus: SplitFocus::Second,
            }),
            second: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    buffer_id: 30,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            focus: SplitFocus::First,
        };
        let (kept, dropped) = tree.close_focused().ok().unwrap();
        assert!(matches!(dropped, Window::Tasks(_)));
        assert!(dropped.buffer_ids().contains(&20));
        assert!(matches!(kept, Window::Horizontal { .. }));
        let ids = kept.buffer_ids();
        assert!(ids.contains(&10));
        assert!(ids.contains(&30));
        assert!(!ids.contains(&20));
    }

    #[test]
    fn close_focused_leaf_returns_err() {
        let leaf = Window::Directory(
            ViewPort::default(),
            ViewPort::default(),
            ViewPort::default(),
        );
        let result = leaf.close_focused();
        assert!(result.is_err());
    }

    #[test]
    fn set_wrap_directory_sets_all_three_viewports() {
        let mut window = Window::Directory(
            ViewPort::default(),
            ViewPort::default(),
            ViewPort::default(),
        );
        assert!(!window.focused_viewport().wrap);

        window.set_wrap(true);
        match &window {
            Window::Directory(parent, current, preview) => {
                assert!(parent.wrap);
                assert!(current.wrap);
                assert!(preview.wrap);
            }
            _ => panic!("expected Directory"),
        }

        window.set_wrap(false);
        match &window {
            Window::Directory(parent, current, preview) => {
                assert!(!parent.wrap);
                assert!(!current.wrap);
                assert!(!preview.wrap);
            }
            _ => panic!("expected Directory"),
        }
    }

    #[test]
    fn set_wrap_tasks_sets_single_viewport() {
        let mut window = Window::Tasks(ViewPort::default());
        window.set_wrap(true);
        assert!(window.focused_viewport().wrap);
        window.set_wrap(false);
        assert!(!window.focused_viewport().wrap);
    }

    #[test]
    fn set_wrap_help_sets_single_viewport() {
        let mut window = Window::Help(ViewPort::default());
        window.set_wrap(true);
        assert!(window.focused_viewport().wrap);
    }

    #[test]
    fn set_wrap_quickfix_sets_single_viewport() {
        let mut window = Window::QuickFix(ViewPort::default());
        window.set_wrap(true);
        assert!(window.focused_viewport().wrap);
    }

    #[test]
    fn set_wrap_in_split_affects_focused_leaf() {
        let mut tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
        };
        tree.set_wrap(true);

        match &tree {
            Window::Horizontal { first, second, .. } => {
                match first.as_ref() {
                    Window::Directory(p, c, pr) => {
                        assert!(!p.wrap);
                        assert!(!c.wrap);
                        assert!(!pr.wrap);
                    }
                    _ => panic!("expected Directory"),
                }
                assert!(second.focused_viewport().wrap);
            }
            _ => panic!("expected Horizontal"),
        }
    }
}
