use super::parse_module;

use crate::chainer::ModuleData;
use crate::proc::{ensure_attached, maps::Maps};

use mlua::{Error as LuaError, Lua, Result as LuaResult, Table, Value};

fn convert_to_lua_table(lua: &Lua, module: &ModuleData) -> LuaResult<Table> {
    let table = lua.create_table()?;
    table.set("name", module.name.clone())?;
    table.set("base", module.start as i64)?;
    table.set("size", (module.end - module.start) as i64)?;
    Ok(table)
}

pub fn load_mod_util_module(lua: &Lua) -> LuaResult<Table> {
    let module = lua.create_table()?;

    module.set(
        "getModuleByName",
        lua.create_function(|lua, module_name: String| {
            let (pid, _) = ensure_attached().map_err(LuaError::external)?;
            let areas = Maps::parse(pid).map_err(LuaError::external)?;
            let filter = vec![module_name.clone()];

            let modules = parse_module(&areas, Some(&filter)).map_err(LuaError::external)?;

            if let Some(m) = modules.into_iter().find(|m| m.name == module_name) {
                Ok(Value::Table(convert_to_lua_table(lua, &m)?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    module.set(
        "enumerate",
        lua.create_function(|lua, ()| {
            let (pid, _) = ensure_attached().map_err(LuaError::external)?;
            let areas = Maps::parse(pid).map_err(LuaError::external)?;
            let modules = parse_module(&areas, None).map_err(LuaError::external)?;

            let result_table = lua.create_table()?;
            for (i, m) in modules.iter().enumerate() {
                result_table.set(i + 1, convert_to_lua_table(lua, m)?)?;
            }
            Ok(result_table)
        })?,
    )?;

    module.set(
        "findModuleByAddress",
        lua.create_function(|lua, address: i64| {
            let (pid, _) = ensure_attached().map_err(LuaError::external)?;
            let areas = Maps::parse(pid).map_err(LuaError::external)?;
            let addr_u64 = address as u64;

            let modules = parse_module(&areas, None).map_err(LuaError::external)?;
            for m in modules {
                if addr_u64 >= m.start && addr_u64 < m.end {
                    return Ok(Value::Table(convert_to_lua_table(lua, &m)?));
                }
            }
            Ok(Value::Nil)
        })?,
    )?;

    Ok(module)
}
