use crate::{action::Action, model::Model, update::current::update_current};
use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{CommandMode, Mode, SearchDirection},
    update::{focus, unfocus},
};

use super::{
    commandline::{
        set_content_status, update_commandline, update_on_mode_change, update_on_modification,
    },
    preview::{set_preview_to_selected, validate_preview_viewport},
    save::persist_path_changes,
    search::search,
};

#[tracing::instrument(skip(model, msg))]
pub fn update_with_buffer_message(model: &mut Model, msg: &BufferMessage) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => {
            match (from, to) {
                (Mode::Command(_), Mode::Command(_))
                | (Mode::Insert, Mode::Insert)
                | (Mode::Navigation, Mode::Navigation)
                | (Mode::Normal, Mode::Normal) => return Vec::new(),
                _ => {}
            }

            model.mode = to.clone();
            model.mode_before = Some(from.clone());

            let mut actions = vec![Action::ModeChanged];
            actions.extend(match from {
                Mode::Command(_) => {
                    unfocus(&mut model.commandline.buffer);
                    update_on_mode_change(model)
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    unfocus(&mut model.files.current.buffer);
                    vec![]
                }
            });

            set_content_status(model);

            actions.extend(match to {
                Mode::Command(_) => {
                    focus(&mut model.commandline.buffer);
                    update_on_mode_change(model)
                }
                Mode::Insert => {
                    focus(&mut model.files.current.buffer);
                    update_current(model, Some(msg));
                    vec![]
                }
                Mode::Navigation => {
                    // TODO: handle file operations: show pending with gray, refresh on operation success
                    // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                    focus(&mut model.files.current.buffer);
                    update_current(model, Some(msg));
                    persist_path_changes(model)
                }
                Mode::Normal => {
                    focus(&mut model.files.current.buffer);
                    update_current(model, Some(msg));
                    vec![]
                }
            });

            actions
        }
        BufferMessage::Modification(repeat, modification) => match model.mode {
            Mode::Command(CommandMode::Command) | Mode::Command(CommandMode::PrintMultiline) => {
                update_on_modification(model, repeat, modification)
            }
            Mode::Command(_) => {
                let actions = update_on_modification(model, repeat, modification);

                let term = model
                    .commandline
                    .buffer
                    .lines
                    .last()
                    .map(|bl| bl.content.clone());

                search(model, term);

                actions
            }
            Mode::Insert | Mode::Normal => {
                update_current(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = set_preview_to_selected(model) {
                    validate_preview_viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(rpt, mtn) => match &model.mode {
            Mode::Command(_) => update_commandline(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                if let CursorDirection::Search(dr) = mtn {
                    let term = model.register.get(&'/');
                    search(model, term);

                    let current_dr = match model.register.get_search_direction() {
                        Some(it) => it,
                        None => return Vec::new(),
                    };

                    let dr = match (dr, current_dr) {
                        (Search::Next, SearchDirection::Down) => Search::Next,
                        (Search::Next, SearchDirection::Up) => Search::Previous,
                        (Search::Previous, SearchDirection::Down) => Search::Previous,
                        (Search::Previous, SearchDirection::Up) => Search::Next,
                    };

                    let msg = BufferMessage::MoveCursor(*rpt, CursorDirection::Search(dr.clone()));
                    update_current(model, Some(&msg));
                } else {
                    update_current(model, Some(msg));
                };

                let mut actions = Vec::new();
                if let Some(path) = set_preview_to_selected(model) {
                    validate_preview_viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
        },
        BufferMessage::MoveViewPort(_) => match model.mode {
            Mode::Command(_) => update_commandline(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                update_current(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = set_preview_to_selected(model) {
                    validate_preview_viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
        },
        BufferMessage::SaveBuffer => persist_path_changes(model),

        BufferMessage::RemoveLine(_)
        | BufferMessage::ResetCursor
        | BufferMessage::SetContent(_)
        | BufferMessage::SetCursorToLineContent(_)
        | BufferMessage::SortContent(_) => unreachable!(),
    }
}
