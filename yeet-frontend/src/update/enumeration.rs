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

    let directories = model.file_buffer.get_mut_directories();
    if let Some((path, state, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
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

        update::update(
            &model.mode,
            &None,
            buffer,
            &BufferMessage::SetContent(content),
        );

        if is_first_changed_event {
            if let Some(selection) = selection {
                if cursor::set_cursor_index(&model.mode, &model.search, buffer, selection) {
                    tracing::trace!("setting cursor index from selection: {:?}", selection);
                    *state = DirectoryBufferState::PartiallyLoaded;
                }
            }
        }
    }

    tracing::trace!(
        "changed enumeration for path {:?} with current directory states: current is {:?}, parent is {:?}, preview is {:?}",
        path,
        model.file_buffer.current.state,
        model.file_buffer.parent.state,
        model.file_buffer.preview.state
    );
}

#[tracing::instrument(skip(model))]
pub fn finished(model: &mut Model, path: &PathBuf, selection: &Option<String>) {
    if model.mode != Mode::Navigation {
        return;
    }

    let directories = model.file_buffer.get_mut_directories();
    if let Some((_, state, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
        update::update(
            &model.mode,
            &model.search,
            buffer,
            &BufferMessage::SortContent(super::SORT),
        );

        if let Some(selection) = selection {
            if !cursor::set_cursor_index(&model.mode, &model.search, buffer, selection) {
                cursor::set_cursor_index_with_history(
                    &model.mode,
                    &model.history,
                    &model.search,
                    buffer,
                    path,
                );
            }
        }

        *state = DirectoryBufferState::Ready;
    }

    tracing::trace!(
        "finished enumeration for path {:?} with current directory states: current is {:?}, parent is {:?}, preview is {:?}",
        path,
        model.file_buffer.current.state,
        model.file_buffer.parent.state,
        model.file_buffer.preview.state
    );
}
