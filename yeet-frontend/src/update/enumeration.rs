use std::path::PathBuf;

use yeet_keymap::message::{ContentKind, Mode};

use crate::{action::Action, model::Model};

use super::{buffer, bufferline, cursor, mark, preview, qfix};

#[tracing::instrument(skip(model, contents))]
pub fn changed(
    model: &mut Model,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
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

        let is_first_changed_event = buffer.lines.is_empty();
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

        if is_first_changed_event {
            if let Some(selection) = selection {
                cursor::set_cursor_index(selection, buffer);
            }
        }
    }

    None
}

#[tracing::instrument(skip(model))]
pub fn finished(model: &mut Model, path: &PathBuf, selection: &Option<String>) {
    if model.mode != Mode::Navigation {
        return;
    }

    let mut buffer = vec![(model.current.path.as_path(), &mut model.current.buffer)];

    if let Some(preview) = &model.preview.path {
        buffer.push((preview, &mut model.preview.buffer));
    }

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
    }

    if let Some((path, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
        super::sort_content(&model.mode, buffer);

        if let Some(selection) = selection {
            if !cursor::set_cursor_index(selection, buffer) {
                cursor::set_cursor_index_with_history(path, &model.history, buffer);
            }
        } else {
            cursor::set_cursor_index_with_history(path, &model.history, buffer);
        }

        preview::selected_path(model);
        preview::viewport(model);
    }
}
