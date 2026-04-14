use std::path::Path;

use yeet_lua::LuaConfiguration;

use crate::model::{App, Window};

use super::app;

pub fn invoke_on_window_change_for_focused(app: &mut App, lua: &LuaConfiguration) {
    let window = match app.current_window() {
        Ok(w) => w,
        Err(_) => return,
    };

    let (parent_id, current_id, preview_id) = match app::get_focused_directory_buffer_ids(window) {
        Some(ids) => ids,
        None => return,
    };

    let parent_path = app
        .contents
        .buffers
        .get(&parent_id)
        .and_then(|buffer| buffer.resolve_path())
        .map(|p| p.to_path_buf());

    let current_path = app
        .contents
        .buffers
        .get(&current_id)
        .and_then(|buffer| buffer.resolve_path())
        .map(|p| p.to_path_buf());

    let preview_path = app
        .contents
        .buffers
        .get(&preview_id)
        .and_then(|buffer| buffer.resolve_path())
        .map(|p| p.to_path_buf());

    let parent_buffer_type = app
        .contents
        .buffers
        .get(&parent_id)
        .map(|b| b.buffer_type_for_lua());

    let current_buffer_type = app
        .contents
        .buffers
        .get(&current_id)
        .map(|b| b.buffer_type_for_lua());

    let preview_buffer_type = app
        .contents
        .buffers
        .get(&preview_id)
        .map(|b| b.buffer_type_for_lua());

    let window = match app.current_window_mut() {
        Ok(w) => w,
        Err(_) => return,
    };

    if let Some((parent, current, preview)) = app::get_focused_directory_viewports_mut(window) {
        yeet_lua::invoke_on_window_change(
            lua,
            [
                parent_path.as_deref(),
                current_path.as_deref(),
                preview_path.as_deref(),
            ],
            &mut [parent, current, preview],
            [parent_buffer_type, current_buffer_type, preview_buffer_type],
        );
    }
}

pub fn on_window_create(lua: &LuaConfiguration, window: &mut Window, path: Option<&Path>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use yeet_buffer::model::viewport::ViewPort;

    fn create_lua_with_hook() -> LuaConfiguration {
        let lua = yeet_lua::Lua::new();
        lua.load(
            r#"
            y = {}
            y.theme = {}
            y.hook = {}
            local hook_mt = {
                __index = {
                    add = function(self, fn_)
                        if type(fn_) == "function" then
                            table.insert(self, fn_)
                        end
                    end
                }
            }
            y.hook.on_window_create = setmetatable({}, hook_mt)
            y.hook.on_window_change = setmetatable({}, hook_mt)
            y.hook.on_window_create:add(function(ctx)
                if ctx.viewport then
                    ctx.viewport.wrap = true
                end
                if ctx.current then
                    ctx.current.wrap = true
                end
            end)
            "#,
        )
        .exec()
        .unwrap();
        lua
    }

    #[test]
    fn hook_preserves_directory_variant() {
        let lua = create_lua_with_hook();
        let mut window = Window::Directory(
            ViewPort::default(),
            ViewPort::default(),
            ViewPort::default(),
        );
        on_window_create(&lua, &mut window, None);
        assert!(matches!(window, Window::Directory(_, _, _)));
    }

    #[test]
    fn hook_preserves_help_variant() {
        let lua = create_lua_with_hook();
        let mut window = Window::Help(ViewPort::default());
        on_window_create(&lua, &mut window, None);
        assert!(matches!(window, Window::Help(_)));
    }

    #[test]
    fn hook_preserves_quickfix_variant() {
        let lua = create_lua_with_hook();
        let mut window = Window::QuickFix(ViewPort::default());
        on_window_create(&lua, &mut window, None);
        assert!(matches!(window, Window::QuickFix(_)));
    }

    #[test]
    fn hook_preserves_tasks_variant() {
        let lua = create_lua_with_hook();
        let mut window = Window::Tasks(ViewPort::default());
        on_window_create(&lua, &mut window, None);
        assert!(matches!(window, Window::Tasks(_)));
    }

    #[test]
    fn on_window_change_sets_preview_prefix_for_directory() {
        let lua = yeet_lua::Lua::new();
        lua.load(
            r#"
            y = {}
            y.theme = {}
            y.hook = {}
            local hook_mt = {
                __index = {
                    add = function(self, fn_)
                        if type(fn_) == "function" then
                            table.insert(self, fn_)
                        end
                    end
                }
            }
            y.hook.on_window_create = setmetatable({}, hook_mt)
            y.hook.on_window_change = setmetatable({}, hook_mt)
            y.hook.on_window_change:add(function(ctx)
                if ctx.preview.buffer_type == "directory" then
                    ctx.preview.prefix_column_width = 2
                else
                    ctx.preview.prefix_column_width = 0
                end
            end)
            "#,
        )
        .exec()
        .unwrap();

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        yeet_lua::invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            [Some("directory"), Some("directory"), Some("directory")],
        );

        assert_eq!(
            preview.prefix_column_width, 2,
            "preview prefix_column_width should be 2 for directory preview"
        );
    }

    #[test]
    fn on_window_change_clears_preview_prefix_for_file() {
        let lua = yeet_lua::Lua::new();
        lua.load(
            r#"
            y = {}
            y.theme = {}
            y.hook = {}
            local hook_mt = {
                __index = {
                    add = function(self, fn_)
                        if type(fn_) == "function" then
                            table.insert(self, fn_)
                        end
                    end
                }
            }
            y.hook.on_window_create = setmetatable({}, hook_mt)
            y.hook.on_window_change = setmetatable({}, hook_mt)
            y.hook.on_window_change:add(function(ctx)
                if ctx.preview.buffer_type == "directory" then
                    ctx.preview.prefix_column_width = 2
                else
                    ctx.preview.prefix_column_width = 0
                end
            end)
            "#,
        )
        .exec()
        .unwrap();

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };

        yeet_lua::invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            [Some("directory"), Some("directory"), Some("content")],
        );

        assert_eq!(
            preview.prefix_column_width, 0,
            "preview prefix_column_width should be 0 for file preview"
        );
    }
}
