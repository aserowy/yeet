use std::path::Path;

use mlua::prelude::*;
use yeet_buffer::model::{viewport::ViewPort, BufferLine};

use crate::viewport::{table_to_viewport, viewport_to_table};

pub fn invoke_on_window_create(
    lua: &crate::LuaConfiguration,
    window_type: &str,
    path: Option<&Path>,
    viewports: &mut [&mut ViewPort],
) {
    if let Err(err) = try_invoke_on_window_create(lua, window_type, path, viewports) {
        tracing::error!("error in y.hook.on_window_create: {:?}", err);
    }
}

fn try_invoke_on_window_create(
    lua: &Lua,
    window_type: &str,
    path: Option<&Path>,
    viewports: &mut [&mut ViewPort],
) -> LuaResult<()> {
    let y: LuaTable = lua.globals().get("y")?;
    let hook: LuaTable = y.get("hook")?;
    let hook_table: LuaTable = hook.get("on_window_create")?;

    let len = hook_table.raw_len();
    if len == 0 {
        return Ok(());
    }

    let ctx = build_context(lua, window_type, path, viewports)?;

    for i in 1..=len {
        let func: LuaValue = hook_table.raw_get(i)?;
        match func {
            LuaValue::Function(f) => {
                if let Err(err) = f.call::<()>(ctx.clone()) {
                    tracing::error!("error in y.hook.on_window_create callback {}: {:?}", i, err);
                }
            }
            _ => {
                tracing::warn!(
                    "y.hook.on_window_create[{}] is not a function, got {:?}",
                    i,
                    func.type_name()
                );
            }
        }
    }

    read_back_context(&ctx, window_type, viewports);

    Ok(())
}

fn build_context(
    lua: &Lua,
    window_type: &str,
    path: Option<&Path>,
    viewports: &mut [&mut ViewPort],
) -> LuaResult<LuaTable> {
    let ctx = lua.create_table()?;
    ctx.set("type", window_type)?;

    if let Some(p) = path {
        ctx.set("path", p.to_string_lossy().to_string())?;
    }

    match window_type {
        "directory" => {
            if viewports.len() == 3 {
                ctx.set("parent", viewport_to_table(lua, viewports[0])?)?;
                ctx.set("current", viewport_to_table(lua, viewports[1])?)?;
                ctx.set("preview", viewport_to_table(lua, viewports[2])?)?;
            }
        }
        _ => {
            if let Some(vp) = viewports.first() {
                ctx.set("viewport", viewport_to_table(lua, vp)?)?;
            }
        }
    }

    Ok(ctx)
}

fn read_back_context(ctx: &LuaTable, window_type: &str, viewports: &mut [&mut ViewPort]) {
    match window_type {
        "directory" => {
            if viewports.len() == 3 {
                if let Ok(t) = ctx.get::<LuaTable>("parent") {
                    table_to_viewport(&t, viewports[0]);
                }
                if let Ok(t) = ctx.get::<LuaTable>("current") {
                    table_to_viewport(&t, viewports[1]);
                }
                if let Ok(t) = ctx.get::<LuaTable>("preview") {
                    table_to_viewport(&t, viewports[2]);
                }
            }
        }
        _ => {
            if let Some(vp) = viewports.first_mut() {
                if let Ok(t) = ctx.get::<LuaTable>("viewport") {
                    table_to_viewport(&t, vp);
                }
            }
        }
    }
}

/// Invokes `y.hook.on_bufferline_mutate` callbacks for a single bufferline.
///
/// Each registered callback receives a context table with:
/// - `buffer`: read-only metadata object containing:
///   - `type`: the buffer type string (e.g., "directory", "content", "help", "quickfix", "tasks")
///   - `path`: the associated path (string) — parent dir for directory buffers, file path for content buffers, empty for others
/// - `prefix`: the bufferline prefix (string or nil), mutable
/// - `content`: the bufferline content as string, mutable
/// - `icon`: the icon glyph (string or nil), mutable
///
/// After all callbacks run, mutable fields are read back from the
/// context table and applied to the bufferline. The `buffer` metadata
/// object is not read back.
pub fn invoke_on_bufferline_mutate(
    lua: &crate::LuaConfiguration,
    bl: &mut BufferLine,
    buffer_type: &str,
    path: &Path,
) {
    if let Err(err) = try_invoke_on_bufferline_mutate(lua, bl, buffer_type, path) {
        tracing::error!("error in y.hook.on_bufferline_mutate: {:?}", err);
    }
}

fn try_invoke_on_bufferline_mutate(
    lua: &Lua,
    bl: &mut BufferLine,
    buffer_type: &str,
    path: &Path,
) -> LuaResult<()> {
    let y: LuaTable = lua.globals().get("y")?;
    let hook: LuaTable = y.get("hook")?;
    let hook_table: LuaTable = hook.get("on_bufferline_mutate")?;

    let len = hook_table.raw_len();
    if len == 0 {
        return Ok(());
    }

    let ctx = lua.create_table()?;

    // Build read-only buffer metadata object
    let buffer_meta = lua.create_table()?;
    buffer_meta.set("type", buffer_type)?;
    buffer_meta.set("path", path.to_string_lossy().to_string())?;
    ctx.set("buffer", buffer_meta)?;

    // Expose full bufferline fields (mutable)
    if let Some(prefix) = &bl.prefix {
        ctx.set("prefix", prefix.as_str())?;
    }
    ctx.set("content", bl.content.to_string())?;
    if let Some(icon) = &bl.icon {
        ctx.set("icon", icon.as_str())?;
    }

    for i in 1..=len {
        let func: LuaValue = hook_table.raw_get(i)?;
        match func {
            LuaValue::Function(f) => {
                if let Err(err) = f.call::<()>(ctx.clone()) {
                    tracing::error!(
                        "error in y.hook.on_bufferline_mutate callback {}: {:?}",
                        i,
                        err
                    );
                }
            }
            _ => {
                tracing::warn!(
                    "y.hook.on_bufferline_mutate[{}] is not a function, got {:?}",
                    i,
                    func.type_name()
                );
            }
        }
    }

    // Read back mutated values
    match ctx.get::<LuaValue>("icon")? {
        LuaValue::String(s) => bl.icon = Some(s.to_str()?.to_string()),
        LuaValue::Nil => bl.icon = None,
        _ => {}
    }
    match ctx.get::<LuaValue>("prefix")? {
        LuaValue::String(s) => bl.prefix = Some(s.to_str()?.to_string()),
        LuaValue::Nil => bl.prefix = None,
        _ => {}
    }
    if let LuaValue::String(s) = ctx.get::<LuaValue>("content")? {
        bl.content = yeet_buffer::model::ansi::Ansi::new(&s.to_str()?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use yeet_buffer::model::viewport::LineNumber;

    fn create_lua_with_hook(script: &str) -> Lua {
        let lua = Lua::new();
        let full_script = format!(
            r#"
            y = {{}}
            y.theme = {{}}
            y.hook = {{}}

            local hook_mt = {{
                __index = {{
                    add = function(self, fn_)
                        if type(fn_) == "function" then
                            table.insert(self, fn_)
                        end
                    end
                }}
            }}
            y.hook.on_window_create = setmetatable({{}}, hook_mt)

            {}
            "#,
            script
        );
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", full_script).unwrap();
        let path = tmp.path().to_path_buf();
        let content = std::fs::read_to_string(&path).unwrap();
        lua.load(&content).exec().unwrap();
        lua
    }

    #[test]
    fn directory_hook_sets_preview_wrap() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.preview.wrap = true
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert!(preview.wrap, "preview.wrap should be true after hook");
    }

    #[test]
    fn directory_hook_sets_parent_wrap() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.wrap = true
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert!(parent.wrap, "parent.wrap should be true after hook");
    }

    #[test]
    fn directory_hook_sets_current_wrap() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.current.wrap = true
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert!(current.wrap, "current.wrap should be true after hook");
    }

    #[test]
    fn directory_hook_sets_preview_wrap_via_real_init() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.preview.wrap = true
                end
            end)
            "#
        )
        .unwrap();

        let lua = Lua::new();
        crate::setup_and_execute(&lua, &tmp.path().to_path_buf()).unwrap();

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert!(
            preview.wrap,
            "preview.wrap should be true after hook via real init"
        );
        assert!(!parent.wrap, "parent.wrap should remain false");
        assert!(!current.wrap, "current.wrap should remain false");
    }

    #[test]
    fn empty_hook_list_is_noop() {
        let lua = create_lua_with_hook("");
        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);
        assert_eq!(vp.line_number, LineNumber::None);
    }

    #[test]
    fn single_callback_modifies_viewport() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "help" then
                    ctx.viewport.wrap = true
                end
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);
        assert!(vp.wrap);
    }

    #[test]
    fn multiple_callbacks_invoked_in_order() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                ctx.viewport.line_number = "absolute"
            end)
            y.hook.on_window_create:add(function(ctx)
                ctx.viewport.wrap = true
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);
        assert_eq!(vp.line_number, LineNumber::Absolute);
        assert!(vp.wrap);
    }

    #[test]
    fn mutations_visible_across_callbacks() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                ctx.viewport.line_number = "absolute"
            end)
            y.hook.on_window_create:add(function(ctx)
                if ctx.viewport.line_number == "absolute" then
                    ctx.viewport.wrap = true
                end
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);
        assert_eq!(vp.line_number, LineNumber::Absolute);
        assert!(vp.wrap);
    }

    #[test]
    fn first_callback_error_does_not_block_second() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                error("boom")
            end)
            y.hook.on_window_create:add(function(ctx)
                ctx.viewport.wrap = true
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);
        assert!(vp.wrap);
    }

    #[test]
    fn all_callbacks_error_preserves_defaults() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                error("error 1")
            end)
            y.hook.on_window_create:add(function(ctx)
                error("error 2")
            end)
            "#,
        );

        let mut vp = ViewPort {
            line_number: LineNumber::Relative,
            ..Default::default()
        };
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);
        assert_eq!(vp.line_number, LineNumber::Relative);
        assert!(!vp.wrap);
    }

    #[test]
    fn directory_hook_with_multiple_callbacks() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.current.line_number = "absolute"
                end
            end)
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.show_border = false
                end
            end)
            "#,
        );

        let mut parent = ViewPort {
            show_border: true,
            ..Default::default()
        };
        let mut current = ViewPort {
            line_number: LineNumber::Relative,
            ..Default::default()
        };
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            Some(Path::new("/tmp")),
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert_eq!(current.line_number, LineNumber::Absolute);
        assert!(!parent.show_border);
    }

    #[test]
    fn hook_receives_path() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.path == "/test/path" then
                    ctx.viewport.wrap = true
                end
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", Some(Path::new("/test/path")), &mut [&mut vp]);
        assert!(vp.wrap);
    }

    #[test]
    fn hook_receives_correct_window_types() {
        for window_type in &["directory", "help", "quickfix", "tasks"] {
            let lua = create_lua_with_hook(&format!(
                r#"
                y.hook.on_window_create:add(function(ctx)
                    if ctx.type == "{}" then
                        if ctx.viewport then
                            ctx.viewport.wrap = true
                        end
                        if ctx.current then
                            ctx.current.wrap = true
                        end
                    end
                end)
                "#,
                window_type
            ));

            if *window_type == "directory" {
                let mut parent = ViewPort::default();
                let mut current = ViewPort::default();
                let mut preview = ViewPort::default();
                invoke_on_window_create(
                    &lua,
                    window_type,
                    None,
                    &mut [&mut parent, &mut current, &mut preview],
                );
                assert!(current.wrap, "wrap should be true for {}", window_type);
            } else {
                let mut vp = ViewPort::default();
                invoke_on_window_create(&lua, window_type, None, &mut [&mut vp]);
                assert!(vp.wrap, "wrap should be true for {}", window_type);
            }
        }
    }

    #[test]
    fn icon_column_width_defaults_to_zero() {
        let vp = ViewPort::default();
        assert_eq!(
            vp.icon_column_width, 0,
            "icon_column_width should default to 0"
        );
    }

    #[test]
    fn directory_hook_sets_icon_column_width_to_one() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.icon_column_width = 1
                    ctx.current.icon_column_width = 1
                    ctx.preview.icon_column_width = 1
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        assert_eq!(parent.icon_column_width, 0);
        assert_eq!(current.icon_column_width, 0);
        assert_eq!(preview.icon_column_width, 0);

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert_eq!(
            parent.icon_column_width, 1,
            "parent icon_column_width should be 1 after hook"
        );
        assert_eq!(
            current.icon_column_width, 1,
            "current icon_column_width should be 1 after hook"
        );
        assert_eq!(
            preview.icon_column_width, 1,
            "preview icon_column_width should be 1 after hook"
        );
    }

    #[test]
    fn icon_column_width_unaffected_for_non_directory_window() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.icon_column_width = 1
                    ctx.current.icon_column_width = 1
                    ctx.preview.icon_column_width = 1
                end
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);

        assert_eq!(
            vp.icon_column_width, 0,
            "icon_column_width should remain 0 for non-directory windows"
        );
    }

    #[test]
    fn icon_column_width_preserved_when_no_hooks() {
        let lua = create_lua_with_hook("");

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert_eq!(
            parent.icon_column_width, 0,
            "icon_column_width should remain 0 with no hooks"
        );
        assert_eq!(
            current.icon_column_width, 0,
            "icon_column_width should remain 0 with no hooks"
        );
        assert_eq!(
            preview.icon_column_width, 0,
            "icon_column_width should remain 0 with no hooks"
        );
    }

    #[test]
    fn icon_column_width_via_real_init() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.icon_column_width = 1
                    ctx.current.icon_column_width = 1
                    ctx.preview.icon_column_width = 1
                end
            end)
            "#
        )
        .unwrap();

        let lua = Lua::new();
        crate::setup_and_execute(&lua, &tmp.path().to_path_buf()).unwrap();

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert_eq!(parent.icon_column_width, 1);
        assert_eq!(current.icon_column_width, 1);
        assert_eq!(preview.icon_column_width, 1);
    }

    #[test]
    fn hook_with_invalid_values_preserves_defaults() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                ctx.viewport.line_number = 42
                ctx.viewport.wrap = "not_bool"
            end)
            "#,
        );

        let mut vp = ViewPort {
            line_number: LineNumber::Relative,
            wrap: true,
            ..Default::default()
        };
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);

        assert_eq!(vp.line_number, LineNumber::Relative);
        assert!(vp.wrap);
    }
}
