use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};
use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    error::AppError,
    model::{history::History, App, Buffer},
    update::app,
};

use super::{hook, selection};

pub fn relocate(
    app: &mut App,
    history: &mut History,
    mode: &Mode,
    direction: &ViewPortDirection,
    lua: Option<&LuaConfiguration>,
) -> Result<Vec<Action>, AppError> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    let (window, contents) = app.current_window_and_contents_mut()?;
    let (vp, focused) = app::get_focused_current_mut(window, contents)?;

    match focused {
        Buffer::Directory(buffer) => {
            yeet_buffer::update(
                Some(vp),
                mode,
                &mut buffer.buffer,
                std::slice::from_ref(&msg),
            );
            let actions = selection::refresh_preview_from_current_selection(app, history, None)?;

            if let Some(lua) = lua {
                hook::invoke_on_window_change_for_focused(app, lua);
            }

            Ok(actions)
        }
        Buffer::Tasks(tasks_buf) => {
            yeet_buffer::update(
                Some(vp),
                mode,
                &mut tasks_buf.buffer,
                std::slice::from_ref(&msg),
            );
            Ok(Vec::new())
        }
        Buffer::QuickFix(qfix_buf) => {
            yeet_buffer::update(
                Some(vp),
                mode,
                &mut qfix_buf.buffer,
                std::slice::from_ref(&msg),
            );
            Ok(Vec::new())
        }
        Buffer::Help(help_buf) => {
            yeet_buffer::update(
                Some(vp),
                mode,
                &mut help_buf.buffer,
                std::slice::from_ref(&msg),
            );
            Ok(Vec::new())
        }
        Buffer::Image(_) | Buffer::Content(_) | Buffer::PathReference(_) | Buffer::Empty => {
            Ok(Vec::new())
        }
    }
}
