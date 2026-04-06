use std::path::Path;

use yeet_lua::Lua;

use crate::model::Window;

pub fn on_window_create(lua: &Lua, window: &mut Window, path: Option<&Path>) {
    match window {
        Window::Directory(parent, current, preview) => {
            yeet_lua::invoke_on_window_create(
                lua,
                "directory",
                path,
                &mut [parent, current, preview],
            );
        }
        Window::Help(vp) => {
            yeet_lua::invoke_on_window_create(lua, "help", None, &mut [vp]);
        }
        Window::QuickFix(vp) => {
            yeet_lua::invoke_on_window_create(lua, "quickfix", None, &mut [vp]);
        }
        Window::Tasks(vp) => {
            yeet_lua::invoke_on_window_create(lua, "tasks", None, &mut [vp]);
        }
        Window::Horizontal { .. } | Window::Vertical { .. } => {}
    }
}
