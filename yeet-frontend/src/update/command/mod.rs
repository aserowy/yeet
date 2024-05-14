use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::Message;

use crate::{
    action::Action,
    model::{mark::Marks, Model},
    task::Task,
    update::command::{
        print::{print_junkyard, print_marks, print_qfix_list, print_register},
        qfix::{
            clear_qfix_list, clear_qfix_list_in_current, do_on_each_qfix_entry,
            invert_qfix_selection_in_current, navigate_first_qfix_entry, navigate_next_qfix_entry,
            navigate_previous_qfix_entry,
        },
    },
};

mod print;
mod qfix;

#[tracing::instrument(skip(model))]
pub fn execute_command(cmd: &str, model: &mut Model) -> Vec<Action> {
    let change_mode_message = Message::Buffer(BufferMessage::ChangeMode(
        model.mode.clone(),
        get_mode_after_command(&model.mode_before),
    ));
    let change_mode_action = Action::EmitMessages(vec![change_mode_message.clone()]);

    let cmd_with_args = match cmd.split_once(' ') {
        Some(it) => it,
        None => (cmd, ""),
    };

    tracing::debug!("executing command: {:?}", cmd_with_args);

    // NOTE: all file commands like e.g. d! should use preview path as target to enable cdo
    match cmd_with_args {
        ("cdo", command) => do_on_each_qfix_entry(model, command, change_mode_action),
        ("cfirst", "") => navigate_first_qfix_entry(model, change_mode_action),
        ("cl", "") => print_qfix_list(&model.qfix),
        ("clearcl", "") => clear_qfix_list_in_current(model, change_mode_action),
        ("clearcl!", "") => clear_qfix_list(model, change_mode_action),
        ("cn", "") => navigate_next_qfix_entry(model, change_mode_action),
        ("cN", "") => navigate_previous_qfix_entry(model, change_mode_action),
        ("cp", target) => {
            let mut actions = vec![change_mode_action];
            if let Some(path) = &model.files.preview.path {
                tracing::info!("copying path: {:?}", path);
                let target = match get_target_file_path(&model.marks, target, path) {
                    Ok(it) => it,
                    Err(err) => {
                        actions.push(Action::EmitMessages(vec![Message::Error(err)]));
                        return actions;
                    }
                };

                actions.push(Action::Task(Task::CopyPath(path.clone(), target)));
            }
            actions
        }
        ("d!", "") => {
            let mut actions = vec![change_mode_action];
            if let Some(path) = &model.files.preview.path {
                tracing::info!("deleting path: {:?}", path);
                actions.push(Action::Task(Task::DeletePath(path.clone())));
            }
            actions
        }
        ("delm", args) if !args.is_empty() => {
            let mut marks = Vec::new();
            for mark in args.chars().filter(|c| c != &' ') {
                marks.push(mark);
            }

            vec![
                change_mode_action,
                Action::EmitMessages(vec![Message::DeleteMarks(marks)]),
            ]
        }
        ("e!", "") => {
            let navigation = if let Some(path) = &model.files.preview.path {
                Message::NavigateToPathAsPreview(path.to_path_buf())
            } else {
                Message::NavigateToPath(model.files.current.path.clone())
            };

            vec![Action::EmitMessages(vec![change_mode_message, navigation])]
        }
        ("invertcl", "") => invert_qfix_selection_in_current(model, change_mode_action),
        ("junk", "") => {
            let content = print_junkyard(&model.junk);
            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("marks", "") => {
            let content = print_marks(&model.marks);
            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("mv", target) => {
            let mut actions = vec![change_mode_action];
            if let Some(path) = &model.files.preview.path {
                tracing::info!("renaming path: {:?}", path);
                let target = match get_target_file_path(&model.marks, target, path) {
                    Ok(it) => it,
                    Err(err) => {
                        actions.push(Action::EmitMessages(vec![Message::Error(err)]));
                        return actions;
                    }
                };

                actions.push(Action::Task(Task::RenamePath(path.clone(), target)));
            }
            actions
        }
        ("noh", "") => vec![Action::EmitMessages(vec![
            change_mode_message,
            Message::ClearSearchHighlight,
        ])],
        ("q", "") => vec![Action::EmitMessages(vec![Message::Quit])],
        ("reg", "") => {
            let content = print_register(&model.register);
            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("w", "") => vec![Action::EmitMessages(vec![
            change_mode_message,
            Message::Buffer(BufferMessage::SaveBuffer),
        ])],
        ("wq", "") => vec![Action::EmitMessages(vec![
            Message::Buffer(BufferMessage::SaveBuffer),
            Message::Quit,
        ])],
        (cmd, args) => {
            let mut actions = vec![change_mode_action];
            if !args.is_empty() {
                let err = format!("command '{} {}' is not valid", cmd, args);
                actions.push(Action::EmitMessages(vec![Message::Error(err)]));
            }
            actions
        }
    }
}

fn get_target_file_path(marks: &Marks, target: &str, path: &Path) -> Result<PathBuf, String> {
    let file_name = match path.file_name() {
        Some(it) => it,
        None => return Err(format!("could not resolve file name from path {:?}", path)),
    };

    let target = if target.starts_with('\'') {
        let mark = match target.chars().nth(1) {
            Some(it) => it,
            None => return Err("invalid mark format".to_string()),
        };

        if let Some(path) = marks.entries.get(&mark) {
            path.to_path_buf()
        } else {
            return Err(format!("mark '{}' not found", mark));
        }
    } else {
        PathBuf::from(target)
    };

    let target_file = target.join(file_name);
    if target.is_dir() && target.exists() && !target_file.exists() {
        Ok(target.join(file_name))
    } else {
        Err("target path is not valid".to_string())
    }
}

fn get_mode_after_command(mode_before: &Option<Mode>) -> Mode {
    if let Some(mode) = mode_before {
        match mode {
            Mode::Command(_) => Mode::default(),
            Mode::Insert | Mode::Normal => Mode::Normal,
            Mode::Navigation => Mode::Navigation,
        }
    } else {
        Mode::default()
    }
}

pub fn create_or_extend_command_stack(model: &mut Model, message: &Message) -> Vec<Action> {
    if let Some(commands) = &mut model.command_stack {
        commands.push_back(message.clone());
    } else {
        let mut stack = VecDeque::new();
        stack.push_back(message.clone());
        model.command_stack = Some(stack);
    }
    Vec::new()
}

mod test {
    #[test]
    fn get_mode_after_command() {
        use yeet_buffer::model::Mode;

        let mode_before = Some(Mode::Normal);
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Normal);

        let mode_before = Some(Mode::Insert);
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Normal);

        let mode_before = Some(Mode::Navigation);
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Navigation);

        let mode_before = None;
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Navigation);
    }
}
