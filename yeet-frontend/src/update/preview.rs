use yeet_buffer::model::{ansi::Ansi, BufferLine, TextBuffer};
use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    event::Preview,
    model::{App, Buffer, ContentBuffer, Contents, PreviewImageBuffer, Window},
    update::app,
};

pub fn update(app: &mut App, lua: Option<&LuaConfiguration>, content: Preview) -> Vec<Action> {
    match content {
        Preview::Content(path, content) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let content: Vec<_> = content
                .iter()
                .map(|s| {
                    let mut line = BufferLine {
                        content: Ansi::new(s),
                        ..Default::default()
                    };
                    if let Some(lua) = lua {
                        yeet_lua::invoke_on_bufferline_mutate(
                            lua,
                            &mut line,
                            yeet_lua::BufferType::Content,
                            Some(&path),
                        );
                    }
                    line
                })
                .collect();

            let (preview_id, _) = app::resolve_buffer(&mut app.contents, &path, &None);
            app.contents.buffers.insert(
                preview_id,
                Buffer::Content(ContentBuffer {
                    path,
                    buffer: TextBuffer::from_lines(content),
                }),
            );
        }
        Preview::Image(path, protocol) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let (preview_id, _) = app::resolve_buffer(&mut app.contents, &path, &None);
            app.contents.buffers.insert(
                preview_id,
                Buffer::Image(PreviewImageBuffer { path, protocol }),
            );
        }
        Preview::None(path) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let (preview_id, _) = app::resolve_buffer(&mut app.contents, &path, &None);
            app.contents
                .buffers
                .insert(preview_id, Buffer::PathReference(path));
        }
    }

    Vec::new()
}

pub fn set_buffer_id(
    contents: &mut Contents,
    window: &mut Window,
    buffer_id: usize,
    lua: Option<&LuaConfiguration>,
) {
    let is_directory = if let Some(Buffer::Directory(it)) = contents.buffers.get(&buffer_id) {
        it.path.is_dir()
    } else {
        false
    };

    if let Some((_, _, preview)) = app::get_focused_directory_viewports_mut(window) {
        preview.buffer_id = buffer_id;
        preview.hide_cursor_line = !is_directory;
    }

    if let Some(lua) = lua {
        let current_path = app::get_focused_directory_viewports(window)
            .and_then(|(_, current_vp, _)| contents.buffers.get(&current_vp.buffer_id))
            .and_then(|buffer| buffer.resolve_path())
            .map(|p| p.to_path_buf());

        if let Some((parent, current, preview)) = app::get_focused_directory_viewports_mut(window) {
            yeet_lua::invoke_on_window_change(
                lua,
                current_path.as_deref(),
                &mut [parent, current, preview],
                is_directory,
            );
        }
    }
}
