use crate::{action::Action, model::Model, update::current};
use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{CommandMode, Mode, SearchDirection},
    update,
};

use super::{commandline, preview, search};

#[tracing::instrument(skip(model, msg))]
pub fn update(model: &mut Model, msg: &BufferMessage) -> Vec<Action> {
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
                    update::unfocus(&mut model.commandline.buffer);
                    commandline::update_on_mode_change(model)
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    update::unfocus(&mut model.files.current.buffer);
                    vec![]
                }
            });

            commandline::set_content_status(model);

            actions.extend(match to {
                Mode::Command(_) => {
                    update::focus(&mut model.commandline.buffer);
                    commandline::update_on_mode_change(model)
                }
                Mode::Insert => {
                    update::focus(&mut model.files.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
                Mode::Navigation => {
                    // TODO: handle file operations: show pending with gray, refresh on operation success
                    // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                    update::focus(&mut model.files.current.buffer);
                    current::update(model, Some(msg));
                    current::save_changes(model)
                }
                Mode::Normal => {
                    update::focus(&mut model.files.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
            });

            actions
        }
        BufferMessage::Modification(repeat, modification) => match model.mode {
            Mode::Command(CommandMode::Command) | Mode::Command(CommandMode::PrintMultiline) => {
                commandline::update_on_modification(model, repeat, modification)
            }
            Mode::Command(_) => {
                let actions = commandline::update_on_modification(model, repeat, modification);

                let search = model
                    .commandline
                    .buffer
                    .lines
                    .last()
                    .map(|bl| bl.content.clone());

                search::search(model, search);

                actions
            }
            Mode::Insert | Mode::Normal => {
                current::update(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = preview::selected_path(model) {
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(rpt, mtn) => match &model.mode {
            Mode::Command(_) => commandline::update(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                if let CursorDirection::Search(dr) = mtn {
                    let search = model.register.get(&'/');
                    search::search(model, search);

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
                    current::update(model, Some(&msg));
                } else {
                    current::update(model, Some(msg));
                };

                let mut actions = Vec::new();
                if let Some(path) = preview::selected_path(model) {
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
        },
        BufferMessage::MoveViewPort(_) => match model.mode {
            Mode::Command(_) => commandline::update(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                current::update(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = preview::selected_path(model) {
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
        },
        BufferMessage::SaveBuffer => current::save_changes(model),

        BufferMessage::RemoveLine(_)
        | BufferMessage::ResetCursor
        | BufferMessage::SetContent(_)
        | BufferMessage::SetCursorToLineContent(_)
        | BufferMessage::SortContent(_) => unreachable!(),
    }
}
