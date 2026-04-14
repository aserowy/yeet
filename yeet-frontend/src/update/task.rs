use std::cmp::Ordering;

use tokio_util::sync::CancellationToken;

use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    model::{Contents, CurrentTask, Tasks, Window},
};

use super::command::task::refresh_tasks_buffer;

pub fn add(
    tasks: &mut Tasks,
    window: &mut Window,
    contents: &mut Contents,
    identifier: String,
    cancellation: CancellationToken,
    lua: Option<&LuaConfiguration>,
) -> Vec<Action> {
    let id = next_id(tasks);

    if let Some(replaced_task) = tasks.running.insert(
        identifier.clone(),
        CurrentTask {
            token: cancellation,
            id,
            external_id: identifier,
        },
    ) {
        replaced_task.token.cancel();
    }

    refresh_tasks_buffer(window, contents, tasks, lua);

    Vec::new()
}

fn next_id(tasks: &mut Tasks) -> u16 {
    let mut next_id = if tasks.latest_id >= 9999 {
        1
    } else {
        tasks.latest_id + 1
    };

    let mut running_ids: Vec<u16> = tasks.running.values().map(|task| task.id).collect();
    running_ids.sort();

    for id in running_ids {
        match next_id.cmp(&id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
        }
    }

    tasks.latest_id = next_id;

    next_id
}

pub fn remove(
    tasks: &mut Tasks,
    window: &mut Window,
    contents: &mut Contents,
    identifier: String,
    lua: Option<&LuaConfiguration>,
) -> Vec<Action> {
    if let Some(task) = tasks.running.remove(&identifier) {
        task.token.cancel();
    }

    refresh_tasks_buffer(window, contents, tasks, lua);

    Vec::new()
}

#[cfg(test)]
mod test {
    use tokio_util::sync::CancellationToken;

    use crate::model::{App, Buffer, CurrentTask, Tasks};
    use crate::update::command::task::open;

    use super::{add, remove};

    fn make_tasks_2() -> Tasks {
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
        tasks
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
            "grep-3".to_string(),
            CurrentTask {
                external_id: "grep baz".to_string(),
                id: 10,
                token: CancellationToken::new(),
            },
        );
        tasks
    }

    fn get_task_line_count(app: &App) -> usize {
        let window = app.current_window().expect("test requires current tab");
        let vp = window.focused_viewport();
        match app.contents.buffers.get(&vp.buffer_id) {
            Some(Buffer::Tasks(tb)) => tb.buffer.lines.len(),
            _ => panic!("expected Buffer::Tasks"),
        }
    }

    fn get_task_line_content(app: &App, index: usize) -> String {
        let window = app.current_window().expect("test requires current tab");
        let vp = window.focused_viewport();
        match app.contents.buffers.get(&vp.buffer_id) {
            Some(Buffer::Tasks(tb)) => tb.buffer.lines[index].content.to_stripped_string(),
            _ => panic!("expected Buffer::Tasks"),
        }
    }

    #[test]
    fn add_refreshes_task_buffer() {
        let tasks = make_tasks_2();
        let mut app = App::default();
        open(&mut app, None, &tasks);

        assert_eq!(get_task_line_count(&app), 2);

        let mut tasks = tasks;
        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        add(
            &mut tasks,
            window,
            contents,
            "grep-3".to_string(),
            CancellationToken::new(),
            None,
        );

        assert_eq!(get_task_line_count(&app), 3);
    }

    #[test]
    fn remove_refreshes_task_buffer() {
        let tasks = make_tasks_2();
        let mut app = App::default();
        open(&mut app, None, &tasks);

        assert_eq!(get_task_line_count(&app), 2);

        let mut tasks = tasks;
        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        remove(&mut tasks, window, contents, "rg-1".to_string(), None);

        assert_eq!(get_task_line_count(&app), 1);
        assert_eq!(get_task_line_content(&app, 0), "5    fd bar");
    }

    #[test]
    fn remove_clamps_cursor_when_past_end() {
        let tasks = make_tasks_2();
        let mut app = App::default();
        open(&mut app, None, &tasks);

        let window = app.current_window_mut().expect("test requires current tab");
        window.focused_viewport_mut().cursor.vertical_index = 1;

        let mut tasks = tasks;
        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        remove(&mut tasks, window, contents, "rg-1".to_string(), None);

        let window = app.current_window().expect("test requires current tab");
        assert_eq!(window.focused_viewport().cursor.vertical_index, 0);

        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        remove(&mut tasks, window, contents, "fd-2".to_string(), None);

        assert_eq!(get_task_line_count(&app), 0);
        let window = app.current_window().expect("test requires current tab");
        assert_eq!(window.focused_viewport().cursor.vertical_index, 0);
    }

    #[test]
    fn add_without_task_window_does_not_panic() {
        let mut tasks = Tasks::default();
        let mut app = App::default();

        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        add(
            &mut tasks,
            window,
            contents,
            "rg-1".to_string(),
            CancellationToken::new(),
            None,
        );
        assert_eq!(tasks.running.len(), 1);
    }

    #[test]
    fn remove_without_task_window_does_not_panic() {
        let mut tasks = make_tasks_2();
        let mut app = App::default();

        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        remove(&mut tasks, window, contents, "rg-1".to_string(), None);
        assert_eq!(tasks.running.len(), 1);
    }

    #[test]
    fn add_cursor_stays_on_same_task() {
        let mut tasks = make_tasks_2();
        let mut app = App::default();
        open(&mut app, None, &tasks);

        let window = app.current_window_mut().expect("test requires current tab");
        window.focused_viewport_mut().cursor.vertical_index = 1;

        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        add(
            &mut tasks,
            window,
            contents,
            "grep-3".to_string(),
            CancellationToken::new(),
            None,
        );

        assert_eq!(get_task_line_count(&app), 3);
        let window = app.current_window().expect("test requires current tab");
        assert_eq!(window.focused_viewport().cursor.vertical_index, 2);
        assert_eq!(get_task_line_content(&app, 2), "5    fd bar");
    }

    #[test]
    fn remove_cursor_stays_on_same_task() {
        let mut tasks = make_tasks_3();
        let mut app = App::default();
        open(&mut app, None, &tasks);

        let window = app.current_window_mut().expect("test requires current tab");
        window.focused_viewport_mut().cursor.vertical_index = 2;

        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        remove(&mut tasks, window, contents, "rg-1".to_string(), None);

        assert_eq!(get_task_line_count(&app), 2);
        let window = app.current_window().expect("test requires current tab");
        assert_eq!(window.focused_viewport().cursor.vertical_index, 1);
        assert_eq!(get_task_line_content(&app, 1), "10   grep baz");
    }
}
