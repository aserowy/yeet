use std::path::PathBuf;

use yeet_keymap::message::{ContentKind, Mode};

use crate::{action::Action, model::Model};

use super::{buffer, bufferline, history, mark, preview, qfix};

#[tracing::instrument(skip(model, contents))]
pub fn changed(
    model: &mut Model,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
) -> Option<Vec<Action>> {
    // TODO: handle unsaved changes
    let mut buffer = vec![(model.current.path.as_path(), &mut model.current.buffer)];

    if let Some(preview) = &model.preview.path {
        buffer.push((preview, &mut model.preview.buffer));
    }

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
    }

    if let Some((path, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
        tracing::trace!("enumeration changed for buffer: {:?}", path);

        let content = contents
            .iter()
            .map(|(knd, cntnt)| {
                let mut line = bufferline::from_enumeration(cntnt, knd);
                mark::set_sign_if_marked(&model.marks, &mut line, &path.join(cntnt));
                qfix::set_sign_if_qfix(&model.qfix, &mut line, &path.join(cntnt));

                line
            })
            .collect();

        buffer::set_content(&model.mode, buffer, content);
    }

    None
}

#[tracing::instrument(skip(model))]
pub fn finished(model: &mut Model, path: &PathBuf) -> Option<Vec<Action>> {
    if model.mode != Mode::Navigation {
        return None;
    }

    let mut buffer = vec![(model.current.path.as_path(), &mut model.current.buffer)];

    if let Some(preview) = &model.preview.path {
        buffer.push((preview, &mut model.preview.buffer));
    }

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
    }

    let mut actions = Vec::new();
    if let Some((path, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
        super::sort_content(&model.mode, buffer);
        history::set_cursor_index(path, &model.history, buffer);

        if let Some(preview_actions) = preview::path(model, true, true) {
            actions.extend(preview_actions);
            preview::viewport(model);
        }
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}
