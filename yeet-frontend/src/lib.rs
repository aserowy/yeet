use std::{env, mem, path::PathBuf};

use action::{Action, ActionResult};
use error::AppError;
use event::{Emitter, Message, MessageSource};
use init::{
    history::load_history_from_file, junkyard::init_junkyard, mark::load_marks_from_file,
    qfix::load_qfix_from_files,
};
use layout::{AppLayout, CommandLineLayout};
use model::{qfix::CdoState, Buffer, FileTreeBuffer, Model};
use settings::Settings;
use task::Task;
use terminal::TerminalWrapper;
use tokio_util::sync::CancellationToken;
use update::update_model;
use view::render_model;

use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{KeymapMessage, PrintContent, QuitMode};

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
    let cancellation = CancellationToken::new();
    let mut terminal = TerminalWrapper::start()?;
    let mut emitter = Emitter::start(cancellation.child_token());

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
                .len_or_default(model.commandline.key_sequence.chars().count()),
        );

        let mut actions_after_update = update_model(&mut model, envelope);

        let buffer = match &mut model.buffer {
            Buffer::FileTree(it) => it,
            Buffer::Text(_) => todo!(),
        };
        actions_after_update.extend(get_watcher_changes(&mut model.watches, buffer));

        let mut preview_action_result = action::preview(
            &mut model,
            &mut emitter,
            &mut terminal,
            actions_after_update,
        )
        .await?;

        if preview_action_result.result != ActionResult::SkipRender {
            render_model(&mut terminal, &model)?;
        }

        preview_action_result
            .remaining_actions
            .extend(get_command_from_stack(
                &mut model,
                &emitter,
                &preview_action_result.remaining_actions,
            ));

        let postview_action_result = action::postview(
            &mut model,
            &mut emitter,
            &mut terminal,
            preview_action_result.remaining_actions,
        )
        .await?;

        if let ActionResult::Quit(mode) = postview_action_result.result {
            match mode {
                QuitMode::FailOnRunningTasks => {
                    if model.current_tasks.is_empty() {
                        break;
                    } else {
                        emitter.run(Task::EmitMessages(vec![Message::Keymap(
                            KeymapMessage::Print(vec![PrintContent::Error(
                                "Failed to quit due to running tasks. Check with :tl and stop with :delt <id>.".to_string(),
                            )]),
                        )]));
                    }
                }
                QuitMode::Force => break,
            };
        }
    }

    emitter.shutdown();
    terminal.shutdown()?;

    Ok(())
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

#[tracing::instrument(skip(buffer))]
fn get_watcher_changes(watches: &mut Vec<PathBuf>, buffer: &FileTreeBuffer) -> Vec<Action> {
    let current = vec![
        Some(buffer.current.path.clone()),
        buffer.parent.resolve_path().map(|p| p.to_path_buf()),
        buffer.preview.resolve_path().map(|p| p.to_path_buf()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut actions = Vec::new();
    for path in watches.iter() {
        if !current.contains(path) {
            actions.push(Action::UnwatchPath(path.clone()));
        }
    }

    for path in &current {
        if !watches.contains(path) {
            actions.push(Action::WatchPath(path.clone()));
        }
    }

    let _ = mem::replace(watches, current);

    if !actions.is_empty() {
        tracing::trace!("watcher changes: {:?}", actions);
    }

    actions
}

#[tracing::instrument(skip(model, emitter))]
fn get_command_from_stack(model: &mut Model, emitter: &Emitter, actions: &[Action]) -> Vec<Action> {
    if model.remaining_keysequence.is_none() && model.qfix.cdo == CdoState::None {
        return Vec::new();
    }

    if !emitter.receiver.is_empty() {
        tracing::debug!(
            "execution canceled: current queued message count is {:?}",
            emitter.receiver.len()
        );
        return Vec::new();
    }

    if actions.iter().any(is_message_queueing) {
        tracing::debug!("execution canceled: actions not empty > {:?}", actions);
        return Vec::new();
    }

    if !model.current_tasks.is_empty() {
        tracing::debug!(
            "execution canceled: not all tasks finished > {:?}",
            model.current_tasks
        );
        return Vec::new();
    }

    if let Some(key_sequence) = model.remaining_keysequence.take() {
        if key_sequence.is_empty() {
            tracing::debug!("remaining key sequence is empty");

            model.remaining_keysequence = None;
        } else {
            tracing::debug!("executing remaining key sequence: {:?}", key_sequence);

            return vec![action::emit_keymap(KeymapMessage::ExecuteKeySequence(
                key_sequence.to_string(),
            ))];
        };
    }

    let (next_state, actions) = match &model.qfix.cdo {
        CdoState::Cnext(command) => (
            CdoState::Cdo(Some(model.qfix.current_index), command.to_owned()),
            vec![action::emit_keymap(KeymapMessage::ExecuteCommandString(
                "cn".to_owned(),
            ))],
        ),
        CdoState::Cdo(old_index, command) => {
            if old_index.is_some_and(|index| index >= model.qfix.current_index) {
                (CdoState::None, Vec::new())
            } else {
                (
                    CdoState::Cnext(command.to_owned()),
                    vec![action::emit_keymap(KeymapMessage::ExecuteCommandString(
                        command.to_owned(),
                    ))],
                )
            }
        }
        CdoState::None => (CdoState::None, Vec::new()),
    };

    tracing::info!("cdo state change: {:?} -> {:?}", model.qfix.cdo, next_state);

    model.qfix.cdo = next_state;

    actions
}

fn is_message_queueing(action: &Action) -> bool {
    match action {
        Action::EmitMessages(_) => true,

        Action::Load(_, _, _)
        | Action::Open(_)
        | Action::Resize(_, _)
        | Action::Task(_)
        | Action::ModeChanged
        | Action::Quit(_, _)
        | Action::UnwatchPath(_)
        | Action::WatchPath(_) => false,
    }
}
