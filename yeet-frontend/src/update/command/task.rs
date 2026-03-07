use std::mem;

use yeet_buffer::model::{viewport::ViewPort, BufferLine, TextBuffer};

use crate::{
    action::Action,
    model::{App, Buffer, Contents, CurrentTask, SplitFocus, Tasks, TasksBuffer, Window},
    update::app,
};

pub fn delete(tasks: &mut Tasks, id: u16) -> Vec<Action> {
    if let Some((_, task)) = tasks.running.iter().find(|(_, task)| task.id == id) {
        task.token.cancel();
    }
    Vec::new()
}

pub fn open(app: &mut App, tasks: &Tasks) -> Vec<Action> {
    if focus_tasks(&mut app.window) {
        return Vec::new();
    }

    let lines = build_task_lines(tasks);
    let buffer_id = app::get_next_buffer_id(&mut app.contents);
    app.contents.buffers.insert(
        buffer_id,
        Buffer::Tasks(TasksBuffer {
            buffer: TextBuffer {
                lines,
                ..Default::default()
            },
        }),
    );

    let task_viewport = ViewPort {
        buffer_id,
        show_border: false,
        ..Default::default()
    };

    let old_window = mem::take(&mut app.window);
    app.window = Window::Horizontal {
        first: Box::new(old_window),
        second: Box::new(Window::Tasks(task_viewport)),
        focus: SplitFocus::Second,
    };

    Vec::new()
}

fn focus_tasks(window: &mut Window) -> bool {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        }
        | Window::Vertical {
            first,
            second,
            focus,
        } => {
            if second.contains_tasks() {
                *focus = SplitFocus::Second;
                focus_tasks(second)
            } else if first.contains_tasks() {
                *focus = SplitFocus::First;
                focus_tasks(first)
            } else {
                false
            }
        }
        Window::Tasks(_) => true,
        Window::Directory(_, _, _) => false,
    }
}

pub fn refresh_tasks_buffer(window: &mut Window, contents: &mut Contents, tasks: &Tasks) {
    let vp = match find_tasks_viewport_mut(window) {
        Some(vp) => vp,
        None => return,
    };

    let buffer_id = vp.buffer_id;
    if let Some(Buffer::Tasks(tasks_buffer)) = contents.buffers.get_mut(&buffer_id) {
        let old_cursor_task_id: Option<u16> = tasks_buffer
            .buffer
            .lines
            .get(vp.cursor.vertical_index)
            .and_then(|line| {
                line.content
                    .to_stripped_string()
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
            });

        tasks_buffer.buffer.lines = build_task_lines(tasks);
        let line_count = tasks_buffer.buffer.lines.len();

        if let Some(old_id) = old_cursor_task_id {
            let mut sorted_tasks: Vec<_> = tasks.running.values().collect();
            sorted_tasks.sort_by_key(|t| t.id);
            if let Some(new_idx) = sorted_tasks.iter().position(|t| t.id == old_id) {
                vp.cursor.vertical_index = new_idx;
                return;
            }
        }

        if vp.cursor.vertical_index >= line_count {
            vp.cursor.vertical_index = line_count.saturating_sub(1);
        }
    }
}

fn find_tasks_viewport_mut(window: &mut Window) -> Option<&mut ViewPort> {
    match window {
        Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
            find_tasks_viewport_mut(first).or_else(|| find_tasks_viewport_mut(second))
        }
        Window::Tasks(vp) => Some(vp),
        Window::Directory(_, _, _) => None,
    }
}

fn build_task_lines(tasks: &Tasks) -> Vec<BufferLine> {
    let mut entries: Vec<_> = tasks.running.values().collect();
    entries.sort_by_key(|task| task.id);
    entries.iter().map(|task| build_task_line(task)).collect()
}

fn build_task_line(task: &CurrentTask) -> BufferLine {
    let formatted = format!("{:<4} {}", task.id, task.external_id);
    if task.token.is_cancelled() {
        BufferLine::from(&format!("\x1b[9;90m{}\x1b[0m", formatted))
    } else {
        BufferLine::from(&formatted)
    }
}

#[cfg(test)]
mod test {
    use tokio_util::sync::CancellationToken;

    use crate::model::{App, Buffer, CurrentTask, SplitFocus, Tasks, Window};

    use super::open;

    fn make_tasks_with_entries() -> Tasks {
        let mut tasks = Tasks::default();
        tasks.running.insert(
            "rg-1".to_string(),
            CurrentTask {
                external_id: "rg foo".to_string(),
                id: 1,
                token: CancellationToken::new(),
            },
        );
        tasks.running.insert(
            "fd-2".to_string(),
            CurrentTask {
                external_id: "fd bar".to_string(),
                id: 12,
                token: CancellationToken::new(),
            },
        );
        tasks
    }

    #[test]
    fn open_creates_horizontal_with_tasks() {
        let mut app = App::default();
        let tasks = Tasks::default();

        open(&mut app, &tasks);

        assert!(matches!(
            app.window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));

        match &app.window {
            Window::Horizontal { first, second, .. } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                assert!(matches!(second.as_ref(), Window::Tasks(_)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_with_tasks_creates_formatted_lines() {
        let mut app = App::default();
        let tasks = make_tasks_with_entries();

        open(&mut app, &tasks);

        let task_vp = match &app.window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::Tasks(vp) => vp,
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Horizontal"),
        };

        let buffer = app.contents.buffers.get(&task_vp.buffer_id).unwrap();
        let lines = match buffer {
            Buffer::Tasks(tb) => &tb.buffer.lines,
            _ => panic!("expected Buffer::Tasks"),
        };

        assert_eq!(lines.len(), 2);
        // Sorted by id: 1 first, then 12.
        assert_eq!(lines[0].content.to_stripped_string(), "1    rg foo");
        assert_eq!(lines[1].content.to_stripped_string(), "12   fd bar");
    }

    #[test]
    fn open_with_no_tasks_creates_empty_buffer() {
        let mut app = App::default();
        let tasks = Tasks::default();

        open(&mut app, &tasks);

        let task_vp = match &app.window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::Tasks(vp) => vp,
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Horizontal"),
        };

        let buffer = app.contents.buffers.get(&task_vp.buffer_id).unwrap();
        match buffer {
            Buffer::Tasks(tb) => assert!(tb.buffer.lines.is_empty()),
            _ => panic!("expected Buffer::Tasks"),
        }
    }

    #[test]
    fn open_idempotent_switches_focus() {
        let mut app = App::default();
        let tasks = Tasks::default();

        open(&mut app, &tasks);
        assert!(matches!(
            app.window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));

        if let Window::Horizontal { focus, .. } = &mut app.window {
            *focus = SplitFocus::First;
        }

        open(&mut app, &tasks);

        match &app.window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert_eq!(*focus, SplitFocus::Second);
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                assert!(matches!(second.as_ref(), Window::Tasks(_)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_registers_buffer_in_contents() {
        let mut app = App::default();
        let tasks = Tasks::default();

        let buffers_before = app.contents.buffers.len();
        open(&mut app, &tasks);

        assert_eq!(app.contents.buffers.len(), buffers_before + 1);

        let task_vp = match &app.window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::Tasks(vp) => vp,
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Horizontal"),
        };

        assert!(app.contents.buffers.contains_key(&task_vp.buffer_id));
        assert!(matches!(
            app.contents.buffers.get(&task_vp.buffer_id),
            Some(Buffer::Tasks(_))
        ));
    }

    #[test]
    fn open_idempotent_focuses_tasks_in_nested_second_child() {
        // Tree: Horizontal { first: Horizontal { first: Dir, second: Tasks }, second: Dir }
        // Both focus fields start at First (pointing away from Tasks).
        use yeet_buffer::model::viewport::ViewPort;

        let mut app = App {
            window: Window::Horizontal {
                first: Box::new(Window::Horizontal {
                    first: Box::new(Window::Directory(
                        ViewPort::default(),
                        ViewPort::default(),
                        ViewPort::default(),
                    )),
                    second: Box::new(Window::Tasks(ViewPort::default())),
                    focus: SplitFocus::First,
                }),
                second: Box::new(Window::Directory(
                    ViewPort::default(),
                    ViewPort::default(),
                    ViewPort::default(),
                )),
                focus: SplitFocus::Second,
            },
            ..Default::default()
        };

        let tasks = Tasks::default();
        open(&mut app, &tasks);

        match &app.window {
            Window::Horizontal { first, focus, .. } => {
                assert_eq!(*focus, SplitFocus::First);
                match first.as_ref() {
                    Window::Horizontal { focus, .. } => {
                        assert_eq!(*focus, SplitFocus::Second);
                    }
                    _ => panic!("expected inner Horizontal"),
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_idempotent_focuses_tasks_in_first_child_of_root() {
        // Tree: Horizontal { first: Tasks, second: Dir, focus: Second }
        use yeet_buffer::model::viewport::ViewPort;

        let mut app = App {
            window: Window::Horizontal {
                first: Box::new(Window::Tasks(ViewPort::default())),
                second: Box::new(Window::Directory(
                    ViewPort::default(),
                    ViewPort::default(),
                    ViewPort::default(),
                )),
                focus: SplitFocus::Second,
            },
            ..Default::default()
        };

        let tasks = Tasks::default();
        open(&mut app, &tasks);

        match &app.window {
            Window::Horizontal { focus, .. } => {
                assert_eq!(*focus, SplitFocus::First);
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_idempotent_focuses_through_three_levels() {
        // Tree: H { first: H { first: H { first: Dir, second: Tasks }, second: Dir }, second: Dir }
        // All focus fields start at Second or First (away from Tasks path).
        use yeet_buffer::model::viewport::ViewPort;

        let dir = || {
            Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )
        };

        let mut app = App {
            window: Window::Horizontal {
                first: Box::new(Window::Horizontal {
                    first: Box::new(Window::Horizontal {
                        first: Box::new(dir()),
                        second: Box::new(Window::Tasks(ViewPort::default())),
                        focus: SplitFocus::First,
                    }),
                    second: Box::new(dir()),
                    focus: SplitFocus::Second,
                }),
                second: Box::new(dir()),
                focus: SplitFocus::Second,
            },
            ..Default::default()
        };

        let tasks = Tasks::default();
        open(&mut app, &tasks);

        match &app.window {
            Window::Horizontal { first, focus, .. } => {
                assert_eq!(*focus, SplitFocus::First, "root");
                match first.as_ref() {
                    Window::Horizontal { first, focus, .. } => {
                        assert_eq!(*focus, SplitFocus::First, "level 2");
                        match first.as_ref() {
                            Window::Horizontal { focus, .. } => {
                                assert_eq!(*focus, SplitFocus::Second, "level 3");
                            }
                            _ => panic!("expected Horizontal at level 3"),
                        }
                    }
                    _ => panic!("expected Horizontal at level 2"),
                }
            }
            _ => panic!("expected Horizontal at root"),
        }
    }

    #[test]
    fn refresh_tasks_buffer_applies_cancelled_styling() {
        let tasks = make_tasks_with_entries();
        let mut app = App::default();
        open(&mut app, &tasks);

        // Cancel the first task (id=1)
        tasks.running.get("rg-1").unwrap().token.cancel();

        super::refresh_tasks_buffer(&mut app.window, &mut app.contents, &tasks);

        let task_vp = match &app.window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::Tasks(vp) => vp,
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Horizontal"),
        };

        let lines = match app.contents.buffers.get(&task_vp.buffer_id).unwrap() {
            Buffer::Tasks(tb) => &tb.buffer.lines,
            _ => panic!("expected Buffer::Tasks"),
        };

        assert_eq!(lines.len(), 2);
        // First line (id=1) is cancelled — has ANSI escapes
        assert_eq!(lines[0].content.to_stripped_string(), "1    rg foo");
        assert!(lines[0].content.to_string().contains("\x1b[9;90m"));
        // Second line (id=12) is not cancelled — plain text
        assert_eq!(lines[1].content.to_stripped_string(), "12   fd bar");
        assert!(!lines[1].content.to_string().contains("\x1b["));
    }

    #[test]
    fn refresh_tasks_buffer_noop_without_tasks_window() {
        let tasks = Tasks::default();
        let mut app = App::default();
        // No :topen — app.window is a plain Directory
        super::refresh_tasks_buffer(&mut app.window, &mut app.contents, &tasks);
        // Should not panic
    }
}
