use std::{env, path::PathBuf};

use action::{Action, ActionResult};
use error::AppError;
use event::{Emitter, Message, MessageSource};
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
use yeet_keymap::message::{KeymapMessage, PrintContent};

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
        Message::Keymap(KeymapMessage::Buffer(BufferMessage::ChangeMode(
            Mode::Normal,
            Mode::default(),
        ))),
        Message::Keymap(KeymapMessage::NavigateToPath(initial_path)),
    ]));

    let mut model = Model {
        settings,
        ..Default::default()
    };

    init_junkyard(&mut model.junk, &mut emitter).await?;

    if load_history_from_file(&mut model.history).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Keymap(
            KeymapMessage::Print(vec![PrintContent::Error(
                "Failed to load history".to_string(),
            )]),
        )]));
    }

    if load_marks_from_file(&mut model.marks).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Keymap(
            KeymapMessage::Print(vec![PrintContent::Error(
                "Failed to load marks".to_string(),
            )]),
        )]));
    }

    if load_qfix_from_files(&mut model.qfix).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Keymap(
            KeymapMessage::Print(vec![PrintContent::Error("Failed to load qfix".to_string())]),
        )]));
    }

    tracing::debug!("starting with model state: {:?}", model);

    let mut result = Vec::new();
    while let Some(envelope) = emitter.receiver.recv().await {
        tracing::debug!("received messages: {:?}", envelope.messages);

        // TODO: C-c should interrupt (clear) cdo commands
        if model.remaining_keysequence.is_some() && envelope.source == MessageSource::User {
            tracing::warn!(
                "skipping user input while cdo commands are running: {:?}",
                envelope.messages
            );

            continue;
        }

        let size = terminal.size().expect("Failed to get terminal size");
        model.layout = AppLayout::new(size, get_commandline_height(&model, &envelope.messages));
        model.commandline.layout = CommandLineLayout::new(
            model.layout.commandline,
            envelope
                .sequence
                .len_or_default(model.key_sequence.chars().count()),
        );

        let mut actions = update_model(&mut model, envelope);
        actions.extend(get_watcher_changes(&mut model));
        actions.extend(get_command_from_stack(&mut model, &actions));

        let exec = action::exec_preview(&mut model, &mut emitter, &mut terminal, actions).await?;
        if exec.result != ActionResult::SkipRender {
            render_model(&mut terminal, &model)?;
        }

        let exec = action::exec_postview(
            &mut model,
            &mut emitter,
            &mut terminal,
            exec.remaining_actions,
        )
        .await?;

        if exec.result == ActionResult::Quit {
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
        if let Message::Keymap(KeymapMessage::Print(content)) = message {
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
        model.files.parent.resolve_path().map(|p| p.to_path_buf()),
        model.files.preview.resolve_path().map(|p| p.to_path_buf()),
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

fn set_remaining_keysequence(model: &mut Model, key_sequence: &str) -> Vec<Action> {
    model.remaining_keysequence = Some(key_sequence.to_owned());

    Vec::new()
}

#[tracing::instrument(skip(model, actions))]
fn get_command_from_stack(model: &mut Model, actions: &[Action]) -> Vec<Action> {
    // Task Start/end Messages Model keeps track
    // Enables :Tasks for overview
    // Enables all Tasks finished

    // Wenn noch Tasks, die relevant sind (mittels flag), aktiv
    if !actions.is_empty() || model.files.current.state != DirectoryBufferState::Ready {
        return Vec::new();
    }

    // Wenn key sequence noch vorhanden, requeue
    if let Some(key_sequence) = model.remaining_keysequence.take() {
        let actions = if key_sequence.is_empty() {
            model.remaining_keysequence = None;
            Vec::new()
        } else {
            vec![Action::EmitMessages(vec![Message::Keymap(
                KeymapMessage::ExecuteKeySequence(key_sequence.to_string()),
            )])]
        };
        return actions;
    }

    // Wenn key sequence none und cdo command gefüllt cnext und warte bis current geladen
    // Führe cdo aus

    Vec::new()
}
