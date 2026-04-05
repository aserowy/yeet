use std::{mem, path::PathBuf};

use yeet_buffer::model::Mode;
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    action::{self, Action},
    error::AppError,
    event::Message,
    model::{self, qfix::QuickFix, App, Buffer, QuickFixBuffer, SplitFocus, Window},
    settings::Settings,
    update::{app, command::qfix::window as qfix_window},
};

pub fn selected(
    settings: &Settings,
    mode: &Mode,
    app: &mut App,
    qfix: &mut QuickFix,
) -> Result<Vec<Action>, AppError> {
    if mode != &Mode::Navigation {
        return Ok(Vec::new());
    }

    let (window, contents) = app.current_window_and_contents_mut()?;
    let (current_vp, current_buffer) = app::get_focused_current_mut(window, contents)?;

    match current_buffer {
        Buffer::QuickFix(qfix_buf) => {
            let cursor_index = current_vp.cursor.vertical_index;
            let path = get_quickfix_entry_path(qfix_buf, cursor_index);
            if let Some(path) = path {
                qfix.current_index = cursor_index;

                let window = app.current_window_mut()?;
                if qfix_window::find_nearest_directory_in_sibling(window).is_some() {
                    qfix_window::focus_nearest_directory(window);
                } else {
                    let empty_buffer = app::get_empty_buffer(&mut app.contents);
                    let new_directory = Window::create(empty_buffer, empty_buffer, empty_buffer);

                    let window = app.current_window_mut()?;
                    let focused_leaf = window.focused_window_mut();
                    let old_window = mem::take(focused_leaf);
                    *focused_leaf = Window::Horizontal {
                        first: Box::new(new_directory),
                        second: Box::new(old_window),
                        focus: SplitFocus::First,
                    };
                }

                return Ok(vec![
                    Action::EmitMessages(vec![Message::QuickFixChanged]),
                    action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(path)),
                ]);
            }
            Ok(Vec::new())
        }
        Buffer::Directory(buffer) => {
            let selected = model::get_selected_path(buffer, &current_vp.cursor);
            if let Some(selected) = selected {
                if settings.selection_to_file_on_open.is_some()
                    || settings.selection_to_stdout_on_open
                {
                    Ok(vec![Action::Quit(
                        QuitMode::FailOnRunningTasks,
                        Some(selected.to_string_lossy().to_string()),
                    )])
                } else {
                    Ok(vec![Action::Open(selected)])
                }
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn get_quickfix_entry_path(qfix_buf: &QuickFixBuffer, cursor_index: usize) -> Option<PathBuf> {
    let line = qfix_buf.buffer.lines.get(cursor_index)?;
    let stripped = line.content.to_stripped_string();
    let path_str = stripped.split_whitespace().nth(1)?;
    Some(PathBuf::from(path_str))
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use yeet_buffer::model::Mode;

    use crate::{
        model::{qfix::QuickFix, App, SplitFocus, Window},
        settings::Settings,
        update::command::qfix::window as qfix_window,
    };

    use super::selected;

    fn make_app_with_standalone_qfix(entries: Vec<PathBuf>) -> (App, QuickFix) {
        let mut app = App::default();
        let qfix = QuickFix {
            current_index: 0,
            entries,
            ..Default::default()
        };
        qfix_window::open(&mut app, &qfix);

        let window = app.current_window_mut().expect("current tab");
        match window {
            Window::Horizontal { second, .. } => {
                let qfix_window = std::mem::take(second.as_mut());
                *window = qfix_window;
            }
            _ => panic!("expected Horizontal after open"),
        }

        (app, qfix)
    }

    fn make_app_with_split_qfix(entries: Vec<PathBuf>) -> (App, QuickFix) {
        let mut app = App::default();
        let qfix = QuickFix {
            current_index: 0,
            entries,
            ..Default::default()
        };
        qfix_window::open(&mut app, &qfix);
        (app, qfix)
    }

    #[test]
    fn enter_on_standalone_qfix_creates_split() {
        let (mut app, mut qfix) = make_app_with_standalone_qfix(vec![PathBuf::from("/tmp/a")]);
        let settings = Settings::default();

        let actions =
            selected(&settings, &Mode::Navigation, &mut app, &mut qfix).expect("should not error");

        assert!(!actions.is_empty(), "should emit navigation action");

        let window = app.current_window().expect("current tab");
        match window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert!(
                    matches!(first.as_ref(), Window::Directory(_, _, _)),
                    "first child should be new Directory"
                );
                assert!(
                    matches!(second.as_ref(), Window::QuickFix(_)),
                    "second child should be QuickFix"
                );
                assert_eq!(
                    *focus,
                    SplitFocus::First,
                    "focus should be on directory (first)"
                );
            }
            _ => panic!("expected Horizontal split"),
        }
    }

    #[test]
    fn enter_on_standalone_qfix_updates_current_index() {
        let (mut app, mut qfix) =
            make_app_with_standalone_qfix(vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]);
        let settings = Settings::default();

        let window = app.current_window_mut().expect("current tab");
        if let Window::QuickFix(vp) = window {
            vp.cursor.vertical_index = 1;
        }

        selected(&settings, &Mode::Navigation, &mut app, &mut qfix).expect("should not error");

        assert_eq!(qfix.current_index, 1);
    }

    #[test]
    fn enter_with_sibling_directory_focuses_directory() {
        let (mut app, mut qfix) = make_app_with_split_qfix(vec![PathBuf::from("/tmp/a")]);
        let settings = Settings::default();

        let actions =
            selected(&settings, &Mode::Navigation, &mut app, &mut qfix).expect("should not error");

        assert!(!actions.is_empty(), "should emit navigation action");

        let window = app.current_window().expect("current tab");
        match window {
            Window::Horizontal { focus, .. } => {
                assert_eq!(
                    *focus,
                    SplitFocus::First,
                    "focus should move to directory (first)"
                );
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn enter_on_empty_qfix_is_noop() {
        let (mut app, mut qfix) = make_app_with_standalone_qfix(vec![]);
        let settings = Settings::default();

        let actions =
            selected(&settings, &Mode::Navigation, &mut app, &mut qfix).expect("should not error");

        assert!(actions.is_empty(), "empty qfix should be a no-op");
    }
}
