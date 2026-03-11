use yeet_buffer::message::{BufferMessage, TextModification};

use crate::{
    action::Action,
    error::AppError,
    model::{App, Buffer, State},
    update::app,
};

use super::{command::task, selection};

pub fn buffer(
    app: &mut App,
    state: &mut State,
    repeat: &usize,
    modification: &TextModification,
) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let (vp, focused) = app::get_focused_current_mut(window, contents)?;
    let result = match focused {
        Buffer::Tasks(_) => {
            if !matches!(modification, TextModification::DeleteLine) {
                return Ok(Vec::new());
            }

            let cursor_index = vp.cursor.vertical_index;
            cancel_task_at_index(&mut state.tasks, cursor_index);

            let (window, contents) = app.current_window_and_contents_mut()?;
            task::refresh_tasks_buffer(window, contents, &state.tasks);

            Vec::new()
        }
        Buffer::Directory(it) => {
            let mode = &state.modes.current;
            let msg = BufferMessage::Modification(*repeat, modification.clone());
            yeet_buffer::update(Some(vp), mode, &mut it.buffer, std::slice::from_ref(&msg));

            let (window, contents) = app.current_window_and_contents_mut()?;
            let (_, buffer) = app::get_focused_current_mut(window, contents)?;
            if matches!(buffer, Buffer::Directory(_)) {
                selection::refresh_preview_from_current_selection(app, &mut state.history, None)?
            } else {
                Vec::new()
            }
        }
        Buffer::Image(_) | Buffer::Content(_) | Buffer::PathReference(_) | Buffer::Empty => {
            Vec::new()
        }
    };
    Ok(result)
}

fn cancel_task_at_index(tasks: &mut crate::model::Tasks, cursor_index: usize) {
    let mut entries: Vec<_> = tasks.running.values_mut().collect();
    entries.sort_by_key(|task| task.id);

    if let Some(task) = entries.get_mut(cursor_index) {
        task.token.cancel();
    }
}

#[cfg(test)]
mod test {
    use tokio_util::sync::CancellationToken;
    use yeet_buffer::{message::TextModification, model::BufferLine};

    use crate::{
        error::AppError,
        model::{App, Buffer, CurrentTask, State, Tasks},
    };

    use super::buffer;

    fn make_app_with_tasks(tasks: &Tasks) -> App {
        use crate::update::command::task::open;

        let mut app = App::default();
        open(&mut app, tasks);
        app
    }

    fn make_tasks_3() -> Tasks {
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
                id: 5,
                token: CancellationToken::new(),
            },
        );
        tasks.running.insert(
            "rg-3".to_string(),
            CurrentTask {
                external_id: "rg baz".to_string(),
                id: 10,
                token: CancellationToken::new(),
            },
        );
        tasks
    }

    #[test]
    fn dd_cancels_task_at_cursor_index() {
        let tasks = make_tasks_3();
        let token_fd = tasks.running.get("fd-2").unwrap().token.clone();
        let mut app = make_app_with_tasks(&tasks);
        let mut state = State {
            tasks,
            ..Default::default()
        };
        state.modes.current = yeet_buffer::model::Mode::Navigation;

        let window = app.current_window_mut().expect("test requires current tab");
        let vp = window.focused_viewport_mut();
        vp.cursor.vertical_index = 1;

        let _ = buffer(&mut app, &mut state, &1, &TextModification::DeleteLine);

        // Task at index 1 (id=5, "fd-2") should be cancelled
        assert!(token_fd.is_cancelled());
        assert!(state
            .tasks
            .running
            .get("fd-2")
            .unwrap()
            .token
            .is_cancelled());

        // Other tasks should NOT be cancelled
        assert!(!state
            .tasks
            .running
            .get("rg-1")
            .unwrap()
            .token
            .is_cancelled());
        assert!(!state
            .tasks
            .running
            .get("rg-3")
            .unwrap()
            .token
            .is_cancelled());
    }

    #[test]
    fn dd_on_tasks_stays_in_navigation_mode() {
        let tasks = make_tasks_3();
        let mut app = make_app_with_tasks(&tasks);
        let mut state = State {
            tasks,
            ..Default::default()
        };
        // mode::change blocks Navigation→Normal on Tasks, so mode stays Navigation.
        state.modes.current = yeet_buffer::model::Mode::Navigation;

        let actions = buffer(&mut app, &mut state, &1, &TextModification::DeleteLine)
            .expect("buffer modification must succeed");

        // Mode should remain Navigation — no mode change occurred.
        assert_eq!(state.modes.current, yeet_buffer::model::Mode::Navigation);
        // No ModeChanged action needed — mode never changed.
        assert!(actions.is_empty());
    }

    #[test]
    fn dd_on_cancelled_task_updates_buffer_with_ansi() {
        let tasks = make_tasks_3();
        let mut app = make_app_with_tasks(&tasks);
        let mut state = State {
            tasks,
            ..Default::default()
        };
        state.modes.current = yeet_buffer::model::Mode::Navigation;

        // Cancel task at index 0 (id=1)
        vp_set_cursor(&mut app, 0);
        let _ = buffer(&mut app, &mut state, &1, &TextModification::DeleteLine);

        // The buffer should be refreshed — cancelled line has ANSI codes
        let lines = get_tasks_buffer_lines(&app).expect("tasks buffer lines");
        assert_eq!(lines.len(), 3);
        // Cancelled line: stripped content is the same, but raw content has ANSI
        assert_eq!(lines[0].content.to_stripped_string(), "1    rg foo");
        assert!(lines[0].content.to_string().contains("\x1b[9;90m"));
        // Non-cancelled lines are plain
        assert!(!lines[1].content.to_string().contains("\x1b["));
        assert!(!lines[2].content.to_string().contains("\x1b["));
    }

    #[test]
    fn dd_on_empty_task_buffer_does_not_panic() {
        let tasks = Tasks::default();
        let mut app = make_app_with_tasks(&tasks);
        let mut state = State::default();
        state.modes.current = yeet_buffer::model::Mode::Navigation;

        let _ = buffer(&mut app, &mut state, &1, &TextModification::DeleteLine);
        // No panic — test passes if we reach here
    }

    #[test]
    fn non_delete_modification_on_tasks_is_blocked() {
        let tasks = make_tasks_3();
        let mut app = make_app_with_tasks(&tasks);
        let mut state = State {
            tasks,
            ..Default::default()
        };
        state.modes.current = yeet_buffer::model::Mode::Navigation;

        // Try inserting text — should be blocked
        buffer(
            &mut app,
            &mut state,
            &1,
            &TextModification::Insert("x".to_string()),
        )
        .expect("buffer modification must succeed");

        // No task should be cancelled
        for task in state.tasks.running.values() {
            assert!(!task.token.is_cancelled());
        }

        // Buffer content should be unchanged (3 plain lines)
        let lines = get_tasks_buffer_lines(&app).expect("tasks buffer lines");
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert!(!line.content.to_string().contains("\x1b["));
        }
    }

    fn vp_set_cursor(app: &mut App, index: usize) {
        let window = app.current_window_mut().expect("test requires current tab");
        let vp = window.focused_viewport_mut();
        vp.cursor.vertical_index = index;
    }

    fn get_tasks_buffer_lines(app: &App) -> Result<Vec<BufferLine>, AppError> {
        let window = app.current_window()?;
        let vp = window.focused_viewport();
        match app.contents.buffers.get(&vp.buffer_id) {
            Some(Buffer::Tasks(tb)) => Ok(tb.buffer.lines.clone()),
            _ => Err(AppError::InvalidState(format!(
                "Expected Tasks buffer at id {}",
                vp.buffer_id,
            ))),
        }
    }
}
