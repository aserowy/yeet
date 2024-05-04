use std::path::PathBuf;

use ratatui::style::Color;
use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, Cursor, CursorPosition, Mode, StylePartial, StylePartialSpan},
    update,
};
use yeet_keymap::message::ContentKind;

use crate::model::{DirectoryBufferState, Model};

use super::{cursor, mark, qfix};

#[tracing::instrument(skip(model, contents))]
pub fn changed(
    model: &mut Model,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) {
    // TODO: handle unsaved changes

    let directories = model.files.get_mut_directories();
    if let Some((path, state, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
        tracing::trace!("enumeration changed for buffer: {:?}", path);

        let is_first_changed_event = buffer.lines.is_empty();
        let content = contents
            .iter()
            .map(|(knd, cntnt)| {
                let mut line = from_enumeration(cntnt, knd);
                mark::set_sign_if_marked(&model.marks, &mut line, &path.join(cntnt));
                qfix::set_sign_if_qfix(&model.qfix, &mut line, &path.join(cntnt));

                line
            })
            .collect();

        update::update(&model.mode, buffer, &BufferMessage::SetContent(content));

        if is_first_changed_event {
            if let Some(selection) = selection {
                if cursor::set_cursor_index(&model.mode, buffer, selection) {
                    tracing::trace!("setting cursor index from selection: {:?}", selection);
                    *state = DirectoryBufferState::PartiallyLoaded;
                }
            }
        }
    }

    tracing::trace!(
        "changed enumeration for path {:?} with current directory states: current is {:?}, parent is {:?}, preview is {:?}",
        path,
        model.files.current.state,
        model.files.parent.state,
        model.files.preview.state
    );
}

#[tracing::instrument(skip(model))]
pub fn finished(model: &mut Model, path: &PathBuf, selection: &Option<String>) {
    if model.mode != Mode::Navigation {
        return;
    }

    let directories = model.files.get_mut_directories();
    if let Some((_, state, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
        update::update(
            &model.mode,
            buffer,
            &BufferMessage::SortContent(super::SORT),
        );

        if let Some(selection) = selection {
            if buffer.cursor.is_none() {
                buffer.cursor = Some(Cursor {
                    horizontal_index: CursorPosition::None,
                    vertical_index: 0,
                    ..Default::default()
                });
            }

            if !cursor::set_cursor_index(&model.mode, buffer, selection) {
                cursor::set_cursor_index_with_history(&model.mode, &model.history, buffer, path);
            }
        }

        *state = DirectoryBufferState::Ready;
    }

    tracing::trace!(
        "finished enumeration for path {:?} with current directory states: current is {:?}, parent is {:?}, preview is {:?}",
        path,
        model.files.current.state,
        model.files.parent.state,
        model.files.preview.state
    );
}

pub fn from_enumeration(content: &String, kind: &ContentKind) -> BufferLine {
    // TODO: refactor with by path
    let style = if kind == &ContentKind::Directory {
        let length = content.chars().count();
        vec![StylePartialSpan {
            end: length,
            style: StylePartial::Foreground(Color::LightBlue),
            ..Default::default()
        }]
    } else {
        vec![]
    };

    BufferLine {
        content: content.to_string(),
        style,
        ..Default::default()
    }
}
