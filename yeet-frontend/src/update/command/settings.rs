use yeet_buffer::model::Mode;

use crate::{action::Action, model::App};

pub fn execute(app: &mut App, args: &str, mode_before: Mode, mode: Mode) -> Vec<Action> {
    match args {
        "wrap" => set_wrap(app, true, mode_before, mode),
        "nowrap" => set_wrap(app, false, mode_before, mode),
        "" => super::print_error("set requires an argument", mode_before, mode),
        arg => super::print_error(&format!("Unknown set option: '{}'", arg), mode_before, mode),
    }
}

fn set_wrap(app: &mut App, wrap: bool, mode_before: Mode, mode: Mode) -> Vec<Action> {
    match app.current_window_mut() {
        Ok(window) => {
            window.set_wrap(wrap);
            super::add_change_mode(mode_before, mode, Vec::new())
        }
        Err(err) => super::print_error(&err.to_string(), mode_before, mode),
    }
}

#[cfg(test)]
mod test {
    use yeet_buffer::model::{CommandMode, Mode};

    use crate::{
        action::Action,
        event::Message,
        model::{App, Window},
    };

    use super::execute;

    fn mode_before() -> Mode {
        Mode::Command(CommandMode::Command)
    }

    fn mode_after() -> Mode {
        Mode::Navigation
    }

    fn contains_error(actions: &[Action], needle: &str) -> bool {
        actions.iter().any(|a| {
            if let Action::EmitMessages(msgs) = a {
                msgs.iter()
                    .any(|m| matches!(m, Message::Error(s) if s.contains(needle)))
            } else {
                false
            }
        })
    }

    #[test]
    fn wrap_enables_wrap_on_directory_window() {
        let mut app = App::default();
        let actions = execute(&mut app, "wrap", mode_before(), mode_after());
        assert!(!contains_error(&actions, ""), "set wrap should not error");

        let window = app.current_window().expect("tab exists");
        match window {
            Window::Directory(parent, current, preview) => {
                assert!(parent.wrap);
                assert!(current.wrap);
                assert!(preview.wrap);
            }
            _ => panic!("expected Directory window"),
        }
    }

    #[test]
    fn nowrap_disables_wrap_on_directory_window() {
        let mut app = App::default();
        execute(&mut app, "wrap", mode_before(), mode_after());
        let actions = execute(&mut app, "nowrap", mode_before(), mode_after());
        assert!(!contains_error(&actions, ""), "set nowrap should not error");

        let window = app.current_window().expect("tab exists");
        match window {
            Window::Directory(parent, current, preview) => {
                assert!(!parent.wrap);
                assert!(!current.wrap);
                assert!(!preview.wrap);
            }
            _ => panic!("expected Directory window"),
        }
    }

    #[test]
    fn unknown_option_returns_error() {
        let mut app = App::default();
        let actions = execute(&mut app, "foobar", mode_before(), mode_after());
        assert!(
            contains_error(&actions, "Unknown set option"),
            "should error on unknown option; actions: {actions:?}"
        );
    }

    #[test]
    fn empty_returns_error() {
        let mut app = App::default();
        let actions = execute(&mut app, "", mode_before(), mode_after());
        assert!(
            contains_error(&actions, "requires an argument"),
            "should error on empty set; actions: {actions:?}"
        );
    }
}
