use std::path::Path;

use mlua::prelude::*;
use yeet_buffer::model::{viewport::ViewPort, BufferLine};

use crate::viewport::{table_to_viewport, viewport_to_table};

/// Represents the type of buffer being mutated.
///
/// Used by `invoke_on_bufferline_mutate` callers to specify the buffer type
/// in a type-safe way. Each variant maps to its lowercase string representation
/// for injection into the Lua hook context (`ctx.buffer.type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Directory,
    Content,
    Help,
    Quickfix,
    Tasks,
}

impl BufferType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BufferType::Directory => "directory",
            BufferType::Content => "content",
            BufferType::Help => "help",
            BufferType::Quickfix => "quickfix",
            BufferType::Tasks => "tasks",
        }
    }
}

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

pub fn invoke_on_window_change(
    lua: &crate::LuaConfiguration,
    viewport_paths: [Option<&Path>; 3],
    viewports: &mut [&mut ViewPort],
    preview_is_directory: bool,
) {
    if let Err(err) =
        try_invoke_on_window_change(lua, viewport_paths, viewports, preview_is_directory)
    {
        tracing::error!("error in y.hook.on_window_change: {:?}", err);
    }
}

fn try_invoke_on_window_change(
    lua: &Lua,
    viewport_paths: [Option<&Path>; 3],
    viewports: &mut [&mut ViewPort],
    preview_is_directory: bool,
) -> LuaResult<()> {
    let y: LuaTable = lua.globals().get("y")?;
    let hook: LuaTable = y.get("hook")?;
    let hook_table: LuaTable = hook.get("on_window_change")?;

    let len = hook_table.raw_len();
    if len == 0 {
        return Ok(());
    }

    let ctx = build_context(lua, "directory", None, viewports)?;
    ctx.set("preview_is_directory", preview_is_directory)?;

    let viewport_keys = ["parent", "current", "preview"];
    for (key, path) in viewport_keys.iter().zip(viewport_paths.iter()) {
        if let Some(p) = path {
            if let Ok(subtable) = ctx.get::<LuaTable>(*key) {
                subtable.set("path", p.to_string_lossy().to_string())?;
            }
        }
    }

    for i in 1..=len {
        let func: LuaValue = hook_table.raw_get(i)?;
        match func {
            LuaValue::Function(f) => {
                if let Err(err) = f.call::<()>(ctx.clone()) {
                    tracing::error!("error in y.hook.on_window_change callback {}: {:?}", i, err);
                }
            }
            _ => {
                tracing::warn!(
                    "y.hook.on_window_change[{}] is not a function, got {:?}",
                    i,
                    func.type_name()
                );
            }
        }
    }

    read_back_context(&ctx, "directory", viewports);

    Ok(())
}

/// Invokes `y.hook.on_bufferline_mutate` callbacks for a single bufferline.
///
/// Each registered callback receives a context table with:
/// - `buffer`: read-only metadata object containing:
///   - `type`: the buffer type string derived from `BufferType` enum
///     (e.g., "directory", "content", "help", "quickfix", "tasks")
///   - `path`: the associated path (string) — only set for buffer types with an associated path
///     (directory, content); absent/nil for help, quickfix, tasks
/// - `prefix`: the bufferline prefix (string or nil), mutable
/// - `content`: the bufferline content as string, mutable
///
/// After all callbacks run, mutable fields are read back from the
/// context table and applied to the bufferline. The `buffer` metadata
/// object is not read back.
pub fn invoke_on_bufferline_mutate(
    lua: &crate::LuaConfiguration,
    bl: &mut BufferLine,
    buffer_type: BufferType,
    path: Option<&Path>,
) {
    if let Err(err) = try_invoke_on_bufferline_mutate(lua, bl, buffer_type, path) {
        tracing::error!("error in y.hook.on_bufferline_mutate: {:?}", err);
    }
}

fn try_invoke_on_bufferline_mutate(
    lua: &Lua,
    bl: &mut BufferLine,
    buffer_type: BufferType,
    path: Option<&Path>,
) -> LuaResult<()> {
    let y: LuaTable = lua.globals().get("y")?;
    let hook: LuaTable = y.get("hook")?;
    let hook_table: LuaTable = hook.get("on_bufferline_mutate")?;

    let len = hook_table.raw_len();
    if len == 0 {
        return Ok(());
    }

    let ctx = lua.create_table()?;

    let buffer_meta = lua.create_table()?;
    buffer_meta.set("type", buffer_type.as_str())?;
    if let Some(p) = path {
        buffer_meta.set("path", p.to_string_lossy().to_string())?;
    }
    ctx.set("buffer", buffer_meta)?;

    if let Some(prefix) = &bl.prefix {
        ctx.set("prefix", prefix.as_str())?;
    }
    ctx.set("content", bl.content.to_string())?;

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
            y.hook.on_window_change = setmetatable({{}}, hook_mt)

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
    fn prefix_column_width_defaults_to_zero() {
        let vp = ViewPort::default();
        assert_eq!(
            vp.prefix_column_width, 0,
            "prefix_column_width should default to 0"
        );
    }

    #[test]
    fn directory_hook_sets_prefix_column_width() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.prefix_column_width = 2
                    ctx.current.prefix_column_width = 2
                    ctx.preview.prefix_column_width = 2
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        assert_eq!(parent.prefix_column_width, 0);
        assert_eq!(current.prefix_column_width, 0);
        assert_eq!(preview.prefix_column_width, 0);

        invoke_on_window_create(
            &lua,
            "directory",
            None,
            &mut [&mut parent, &mut current, &mut preview],
        );

        assert_eq!(
            parent.prefix_column_width, 2,
            "parent prefix_column_width should be 2 after hook"
        );
        assert_eq!(
            current.prefix_column_width, 2,
            "current prefix_column_width should be 2 after hook"
        );
        assert_eq!(
            preview.prefix_column_width, 2,
            "preview prefix_column_width should be 2 after hook"
        );
    }

    #[test]
    fn prefix_column_width_unaffected_for_non_directory_window() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.prefix_column_width = 2
                    ctx.current.prefix_column_width = 2
                    ctx.preview.prefix_column_width = 2
                end
            end)
            "#,
        );

        let mut vp = ViewPort::default();
        invoke_on_window_create(&lua, "help", None, &mut [&mut vp]);

        assert_eq!(
            vp.prefix_column_width, 0,
            "prefix_column_width should remain 0 for non-directory windows"
        );
    }

    #[test]
    fn prefix_column_width_preserved_when_no_hooks() {
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
            parent.prefix_column_width, 0,
            "prefix_column_width should remain 0 with no hooks"
        );
        assert_eq!(
            current.prefix_column_width, 0,
            "prefix_column_width should remain 0 with no hooks"
        );
        assert_eq!(
            preview.prefix_column_width, 0,
            "prefix_column_width should remain 0 with no hooks"
        );
    }

    #[test]
    fn prefix_column_width_via_real_init() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.parent.prefix_column_width = 2
                    ctx.current.prefix_column_width = 2
                    ctx.preview.prefix_column_width = 2
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

        assert_eq!(parent.prefix_column_width, 2);
        assert_eq!(current.prefix_column_width, 2);
        assert_eq!(preview.prefix_column_width, 2);
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

    #[test]
    fn on_window_change_callback_invocation() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_change:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.preview.wrap = true
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        assert!(
            preview.wrap,
            "preview.wrap should be true after on_window_change"
        );
    }

    #[test]
    fn on_window_change_viewport_read_back() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_change:add(function(ctx)
                ctx.preview.prefix_column_width = 2
                ctx.parent.line_number = "absolute"
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        assert_eq!(preview.prefix_column_width, 2);
        assert_eq!(parent.line_number, LineNumber::Absolute);
    }

    #[test]
    fn on_window_change_preview_is_directory_true() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_change:add(function(ctx)
                if ctx.preview_is_directory then
                    ctx.preview.prefix_column_width = 2
                else
                    ctx.preview.prefix_column_width = 0
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        assert_eq!(
            preview.prefix_column_width, 2,
            "prefix_column_width should be 2 when preview_is_directory is true"
        );
    }

    #[test]
    fn on_window_change_preview_is_directory_false() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_change:add(function(ctx)
                if ctx.preview_is_directory then
                    ctx.preview.prefix_column_width = 2
                else
                    ctx.preview.prefix_column_width = 0
                end
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };

        invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            false,
        );

        assert_eq!(
            preview.prefix_column_width, 0,
            "prefix_column_width should be 0 when preview_is_directory is false"
        );
    }

    #[test]
    fn on_window_change_error_handling() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_change:add(function(ctx)
                error("boom")
            end)
            y.hook.on_window_change:add(function(ctx)
                ctx.preview.wrap = true
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        assert!(
            preview.wrap,
            "second callback should still run after first errors"
        );
    }

    #[test]
    fn on_window_change_no_callbacks_is_noop() {
        let lua = create_lua_with_hook("");

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };

        invoke_on_window_change(
            &lua,
            [None, None, None],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        assert_eq!(
            preview.prefix_column_width, 2,
            "prefix_column_width should remain unchanged with no callbacks"
        );
    }

    #[test]
    fn on_window_change_per_viewport_paths() {
        let lua = create_lua_with_hook(
            r#"
            y.hook.on_window_change:add(function(ctx)
                _G.test_parent_path = ctx.parent.path
                _G.test_current_path = ctx.current.path
                _G.test_preview_path = ctx.preview.path
                _G.test_top_level_path = ctx.path
            end)
            "#,
        );

        let mut parent = ViewPort::default();
        let mut current = ViewPort::default();
        let mut preview = ViewPort::default();

        invoke_on_window_change(
            &lua,
            [
                Some(Path::new("/parent")),
                Some(Path::new("/current")),
                Some(Path::new("/preview")),
            ],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        let globals = lua.globals();
        assert_eq!(
            globals.get::<String>("test_parent_path").unwrap(),
            "/parent"
        );
        assert_eq!(
            globals.get::<String>("test_current_path").unwrap(),
            "/current"
        );
        assert_eq!(
            globals.get::<String>("test_preview_path").unwrap(),
            "/preview"
        );
        assert!(globals.get::<LuaValue>("test_top_level_path").unwrap() == LuaValue::Nil);
    }

    #[test]
    fn on_window_change_via_real_init() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
            y.hook.on_window_change:add(function(ctx)
                if ctx.preview_is_directory then
                    ctx.preview.prefix_column_width = 2
                else
                    ctx.preview.prefix_column_width = 0
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

        invoke_on_window_change(
            &lua,
            [
                Some(Path::new("/parent")),
                Some(Path::new("/current")),
                Some(Path::new("/preview")),
            ],
            &mut [&mut parent, &mut current, &mut preview],
            true,
        );

        assert_eq!(preview.prefix_column_width, 2);
    }
}
