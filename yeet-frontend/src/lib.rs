use std::{env, path::PathBuf};

use action::{exec_postview_actions, exec_preview_actions, Action, ActionResult};
use error::AppError;
use event::Emitter;
use init::{
    history::load_history_from_file, junkyard::init_junkyard, mark::load_marks_from_file,
    qfix::load_qfix_from_files,
};
use layout::{AppLayout, CommandLineLayout};
use model::{DirectoryBufferState, Model};
use settings::Settings;
use task::Task;
use terminal::TerminalWrapper;
use update::update_model;
use view::render_model;

use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{Envelope, KeySequence, Message, MessageSource, PrintContent};

mod action;
pub mod error;
mod event;
mod init;
mod layout;
mod model;
mod open;
pub mod settings;
mod task;
mod terminal;
mod update;
mod view;

pub async fn run(settings: Settings) -> Result<(), AppError> {
    let mut terminal = TerminalWrapper::start()?;
    let mut emitter = Emitter::start();

    let initial_path = get_initial_path(&settings.startup_path);
    emitter.run(Task::EmitMessages(vec![
        Message::Buffer(BufferMessage::ChangeMode(Mode::Normal, Mode::default())),
        Message::NavigateToPath(initial_path),
    ]));

    let mut model = Model {
        settings,
        ..Default::default()
    };

    init_junkyard(&mut model.junk, &mut emitter).await?;

    if load_history_from_file(&mut model.history).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Print(vec![
            PrintContent::Error("Failed to load history".to_string()),
        ])]));
    }

    if load_marks_from_file(&mut model.marks).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Print(vec![
            PrintContent::Error("Failed to load marks".to_string()),
        ])]));
    }

    if load_qfix_from_files(&mut model.qfix).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Print(vec![
            PrintContent::Error("Failed to load qfix".to_string()),
        ])]));
    }

    let mut result = Vec::new();
    while let Some(envelope) = emitter.receiver.recv().await {
        tracing::debug!("received messages: {:?}", envelope.messages);

        // TODO: C-c should interrupt (clear) cdo commands
        if model.command_stack.is_some() && envelope.source == MessageSource::User {
            tracing::warn!(
                "skipping user input while cdo commands are running: {:?}",
                envelope.messages
            );

            continue;
        }

        let size = terminal.size().expect("Failed to get terminal size");
        model.layout = AppLayout::new(size, get_commandline_height(&model, &envelope.messages));

        let sequence_len = match &envelope.sequence {
            KeySequence::Completed(_) => 0,
            KeySequence::Changed(sequence) => sequence.chars().count() as u16,
            KeySequence::None => model.key_sequence.chars().count() as u16,
        };
        model.commandline.layout = CommandLineLayout::new(model.layout.commandline, sequence_len);

        let mut actions = update_model(&mut model, &envelope);
        actions.extend(get_watcher_changes(&mut model));
        actions.extend(get_command_from_stack(&mut model, &actions, &envelope));

        let result = exec_preview_actions(&model, &mut emitter, &mut terminal, &actions).await?;
        if result != ActionResult::SkipRender {
            render_model(&mut terminal, &model)?;
        }

        let result = exec_postview_actions(&model, &mut emitter, &mut terminal, &actions).await?;
        if result == ActionResult::Quit {
            break;
        }
    }

    if let Err(error) = emitter.shutdown().await {
        result.push(error);
    }

    terminal.shutdown()?;

    if result.is_empty() {
        Ok(())
    } else {
        Err(AppError::Aggregate(result))
    }
}

fn get_initial_path(initial_selection: &Option<PathBuf>) -> PathBuf {
    if let Some(path) = initial_selection {
        if path.exists() {
            return path.to_path_buf();
        }
    }
    env::current_dir().expect("Failed to get current directory")
}

fn get_commandline_height(model: &Model, messages: &Vec<Message>) -> u16 {
    let lines_len = model.commandline.buffer.lines.len();
    let mut height = if lines_len == 0 { 1 } else { lines_len as u16 };
    for message in messages {
        if let Message::Print(content) = message {
            if content.len() > 1 {
                height = content.len() as u16 + 1;
            }
        }
    }
    height
}

#[tracing::instrument(skip(model))]
fn get_watcher_changes(model: &mut Model) -> Vec<Action> {
    let current = vec![
        Some(model.files.current.path.clone()),
        model.files.preview.path.clone(),
        model.files.parent.path.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut actions = Vec::new();
    for path in &model.watches {
        if !current.contains(path) {
            actions.push(Action::UnwatchPath(path.clone()));
        }
    }

    for path in &current {
        if !model.watches.contains(path) {
            actions.push(Action::WatchPath(path.clone()));
        }
    }

    model.watches = current;

    if !actions.is_empty() {
        tracing::trace!("watcher changes: {:?}", actions);
    }

    actions
}

#[tracing::instrument(skip(model, actions))]
fn get_command_from_stack(
    model: &mut Model,
    actions: &[Action],
    envelope: &Envelope,
) -> Vec<Action> {
    if let Some(current) = &model.command_current {
        if envelope.messages.iter().any(|msg| msg == current) {
            model.command_current = None;
        }
    }

    if let Some(commands) = &mut model.command_stack {
        let buffer_loading = model
            .files
            .get_mut_directories()
            .iter()
            .any(|(_, state, _)| state != &&DirectoryBufferState::Ready);

        let current_command_processing = model.command_current.is_some();

        let contains_emit_messages = actions
            .iter()
            .any(|msg| matches!(msg, Action::EmitMessages(_)));

        if buffer_loading || current_command_processing || contains_emit_messages {
            tracing::trace!(
                "cdo commands skipped: buffer loading {:?}, emitting messages {:?}",
                buffer_loading,
                contains_emit_messages
            );

            return Vec::new();
        }

        let mut actions = Vec::new();
        let command = if let Some(Message::NavigateToPathAsPreview(_)) = commands.front() {
            while let Some(front) = commands.front() {
                if let Message::NavigateToPathAsPreview(path) = front {
                    if path.exists() {
                        break;
                    } else {
                        let command = commands.pop_front();
                        tracing::warn!("removing non existing cdo path: {:?}", command);
                    }
                } else {
                    let command = commands.pop_front();
                    tracing::trace!("removing command for non existing path: {:?}", command);
                }
            }

            commands.pop_front()
        } else {
            commands.pop_front()
        };

        if let Some(command) = command {
            tracing::debug!("emitting cdo command: {:?}", command);
            model.command_current = Some(command.clone());
            actions.push(Action::EmitMessages(vec![command]));
        }

        if commands.is_empty() {
            tracing::trace!("cdo commands finished");
            model.command_stack = None;
        }
        actions
    } else {
        Vec::new()
    }
}
