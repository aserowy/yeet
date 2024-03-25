use std::path::PathBuf;

use yeet_buffer::{message::BufferMessage, model::Mode, update};
use yeet_keymap::message::ContentKind;

use crate::model::{DirectoryBufferState, Model};

use super::{bufferline, cursor, mark, qfix};

#[tracing::instrument(skip(model, contents))]
pub fn changed(
    model: &mut Model,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) {
    // TODO: handle unsaved changes
    let marks = model.marks.clone();
    let mode = model.mode.clone();
    let qfix = model.qfix.clone();

    let directories = model.get_mut_directories();
    if let Some((path, state, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
        tracing::trace!("enumeration changed for buffer: {:?}", path);

        let is_first_changed_event = buffer.lines.is_empty();
        let content = contents
            .iter()
            .map(|(knd, cntnt)| {
                let mut line = bufferline::from_enumeration(cntnt, knd);
                mark::set_sign_if_marked(&marks, &mut line, &path.join(cntnt));
                qfix::set_sign_if_qfix(&qfix, &mut line, &path.join(cntnt));

                line
            })
            .collect();

        update::update(&mode, &None, buffer, &BufferMessage::SetContent(content));

        if is_first_changed_event {
            if let Some(selection) = selection {
                if cursor::set_cursor_index(selection, buffer) {
                    tracing::trace!("setting cursor index from selection: {:?}", selection);
                    *state = DirectoryBufferState::PartiallyLoaded;
                }
            }
        }
    }

    tracing::trace!(
        "changed enumeration for path {:?} with current directory states: current is {:?}, parent is {:?}, preview is {:?}",
        path,
        model.current.state,
        model.parent.state,
        model.preview.state
    );
}

#[tracing::instrument(skip(model))]
pub fn finished(model: &mut Model, path: &PathBuf, selection: &Option<String>) {
    if model.mode != Mode::Navigation {
        return;
    }

    // FIX: anti pattern...
    let mode = model.mode.clone();
    let history = model.history.clone();
    let search = model.search.clone();

    let directories = model.get_mut_directories();
    if let Some((_, state, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
        update::update(
            &mode,
            &search,
            buffer,
            &BufferMessage::SortContent(super::SORT),
        );

        if let Some(selection) = selection {
            if !cursor::set_cursor_index(selection, buffer) {
                cursor::set_cursor_index_with_history(path, &history, buffer);
            }
        }

        *state = DirectoryBufferState::Ready;
    }

    tracing::trace!(
        "finished enumeration for path {:?} with current directory states: current is {:?}, parent is {:?}, preview is {:?}",
        path,
        model.current.state,
        model.parent.state,
        model.preview.state
    );
}
