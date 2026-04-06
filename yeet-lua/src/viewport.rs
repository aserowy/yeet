use mlua::prelude::*;
use yeet_buffer::model::viewport::{LineNumber, ViewPort};

pub fn viewport_to_table(lua: &Lua, vp: &ViewPort) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("line_number", line_number_to_str(&vp.line_number))?;
    table.set("line_number_width", vp.line_number_width)?;
    table.set("sign_column_width", vp.sign_column_width)?;
    table.set("show_border", vp.show_border)?;
    table.set("hide_cursor", vp.hide_cursor)?;
    table.set("hide_cursor_line", vp.hide_cursor_line)?;
    table.set("wrap", vp.wrap)?;
    Ok(table)
}

pub fn table_to_viewport(table: &LuaTable, vp: &mut ViewPort) {
    if let Ok(val) = table.get::<String>("line_number") {
        match str_to_line_number(&val) {
            Some(ln) => vp.line_number = ln,
            None => tracing::warn!("unrecognized line_number value: '{}'", val),
        }
    } else if let Ok(val) = table.get::<LuaValue>("line_number") {
        if !matches!(val, LuaValue::Nil) {
            tracing::warn!("invalid type for line_number, expected string");
        }
    }

    read_usize_field(table, "line_number_width", &mut vp.line_number_width);
    read_usize_field(table, "sign_column_width", &mut vp.sign_column_width);
    read_bool_field(table, "show_border", &mut vp.show_border);
    read_bool_field(table, "hide_cursor", &mut vp.hide_cursor);
    read_bool_field(table, "hide_cursor_line", &mut vp.hide_cursor_line);
    read_bool_field(table, "wrap", &mut vp.wrap);
}

fn read_bool_field(table: &LuaTable, key: &str, target: &mut bool) {
    match table.get::<bool>(key) {
        Ok(val) => *target = val,
        Err(_) => {
            if let Ok(val) = table.get::<LuaValue>(key) {
                if !matches!(val, LuaValue::Nil) {
                    tracing::warn!("invalid type for {}, expected boolean", key);
                }
            }
        }
    }
}

fn read_usize_field(table: &LuaTable, key: &str, target: &mut usize) {
    match table.get::<i64>(key) {
        Ok(val) => {
            if val >= 0 {
                *target = val as usize;
            } else {
                tracing::warn!("negative value for {}: {}", key, val);
            }
        }
        Err(_) => {
            if let Ok(val) = table.get::<LuaValue>(key) {
                if !matches!(val, LuaValue::Nil) {
                    tracing::warn!("invalid type for {}, expected integer", key);
                }
            }
        }
    }
}

fn line_number_to_str(ln: &LineNumber) -> &'static str {
    match ln {
        LineNumber::Absolute => "absolute",
        LineNumber::None => "none",
        LineNumber::Relative => "relative",
    }
}

fn str_to_line_number(s: &str) -> Option<LineNumber> {
    match s {
        "absolute" => Some(LineNumber::Absolute),
        "none" => Some(LineNumber::None),
        "relative" => Some(LineNumber::Relative),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lua() -> Lua {
        Lua::new()
    }

    #[test]
    fn round_trip_viewport_defaults() {
        let lua = make_lua();
        let vp = ViewPort::default();
        let table = viewport_to_table(&lua, &vp).unwrap();
        let mut result = ViewPort::default();
        table_to_viewport(&table, &mut result);

        assert_eq!(result.line_number, vp.line_number);
        assert_eq!(result.line_number_width, vp.line_number_width);
        assert_eq!(result.sign_column_width, vp.sign_column_width);
        assert_eq!(result.show_border, vp.show_border);
        assert_eq!(result.hide_cursor, vp.hide_cursor);
        assert_eq!(result.hide_cursor_line, vp.hide_cursor_line);
        assert_eq!(result.wrap, vp.wrap);
    }

    #[test]
    fn round_trip_viewport_custom_values() {
        let lua = make_lua();
        let vp = ViewPort {
            line_number: LineNumber::Absolute,
            line_number_width: 5,
            sign_column_width: 3,
            show_border: true,
            hide_cursor: true,
            hide_cursor_line: true,
            wrap: true,
            ..Default::default()
        };
        let table = viewport_to_table(&lua, &vp).unwrap();
        let mut result = ViewPort::default();
        table_to_viewport(&table, &mut result);

        assert_eq!(result.line_number, LineNumber::Absolute);
        assert_eq!(result.line_number_width, 5);
        assert_eq!(result.sign_column_width, 3);
        assert!(result.show_border);
        assert!(result.hide_cursor);
        assert!(result.hide_cursor_line);
        assert!(result.wrap);
    }

    #[test]
    fn invalid_type_preserves_default() {
        let lua = make_lua();
        let table = lua.create_table().unwrap();
        table.set("line_number", 42).unwrap();
        table.set("wrap", "not_a_bool").unwrap();
        table.set("sign_column_width", "not_a_number").unwrap();

        let mut vp = ViewPort {
            line_number: LineNumber::Relative,
            wrap: true,
            sign_column_width: 2,
            ..Default::default()
        };
        table_to_viewport(&table, &mut vp);

        assert_eq!(vp.line_number, LineNumber::Relative);
        assert!(vp.wrap);
        assert_eq!(vp.sign_column_width, 2);
    }

    #[test]
    fn unrecognized_enum_preserves_default() {
        let lua = make_lua();
        let table = lua.create_table().unwrap();
        table.set("line_number", "fancy").unwrap();

        let mut vp = ViewPort {
            line_number: LineNumber::Relative,
            ..Default::default()
        };
        table_to_viewport(&table, &mut vp);

        assert_eq!(vp.line_number, LineNumber::Relative);
    }

    #[test]
    fn unknown_keys_ignored() {
        let lua = make_lua();
        let table = lua.create_table().unwrap();
        table.set("unknown_field", true).unwrap();
        table.set("another_unknown", 42).unwrap();

        let mut vp = ViewPort::default();
        let original = vp.clone();
        table_to_viewport(&table, &mut vp);

        assert_eq!(vp.line_number, original.line_number);
        assert_eq!(vp.wrap, original.wrap);
    }

    #[test]
    fn line_number_enum_conversion() {
        assert_eq!(line_number_to_str(&LineNumber::Absolute), "absolute");
        assert_eq!(line_number_to_str(&LineNumber::None), "none");
        assert_eq!(line_number_to_str(&LineNumber::Relative), "relative");

        assert_eq!(str_to_line_number("absolute"), Some(LineNumber::Absolute));
        assert_eq!(str_to_line_number("none"), Some(LineNumber::None));
        assert_eq!(str_to_line_number("relative"), Some(LineNumber::Relative));
        assert_eq!(str_to_line_number("invalid"), None);
    }
}
