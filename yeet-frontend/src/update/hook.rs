use std::path::Path;

use yeet_lua::LuaConfiguration;

use crate::model::Window;

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
}
