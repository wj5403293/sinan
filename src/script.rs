mod mem_view;
mod mod_util;

use crate::chainer::{ModuleData, RangeType};
use crate::constants::{APP_ROOT, DIR_SCRIPTS};
use crate::proc::{self, maps::VMArea};

use anyhow::{Result, anyhow};
use mlua::{
    Lua, LuaSerdeExt,
    Value::{self, Table},
};
use std::fs;
use std::path::Path;

fn init_lua() -> Result<Lua> {
    let scripts: [(&str, &str); 3] = [
        ("main.lua", include_str!("script/luas/main.lua")),
        (
            "parse_module.lua",
            include_str!(concat!(
                "script/luas/",
                env!("SCRIPT_PLATFORM"),
                "/parse_module.lua"
            )),
        ),
        (
            "parse_regions.lua",
            include_str!(concat!(
                "script/luas/",
                env!("SCRIPT_PLATFORM"),
                "/parse_regions.lua"
            )),
        ),
    ];

    let script_dir = Path::new(APP_ROOT).join(DIR_SCRIPTS);
    for (filename, content) in &scripts {
        let filepath = script_dir.join(filename);
        if !filepath.exists() {
            fs::write(&filepath, content)?;
        }
    }

    let lua = Lua::new();
    let globals = lua.globals();

    let module = lua.create_table()?;
    globals.set("os", env!("OS"))?;
    globals.set("arch", env!("ARCH"))?;
    module.set("pid", lua.create_function(|_, ()| Ok(proc::get_pid()))?)?;
    module.set(
        "name",
        lua.create_function(|_, ()| Ok(proc::get_target_name()))?,
    )?;

    globals.set("env", module)?;

    Ok(lua)
}

pub fn lua_start() -> Result<()> {
    let lua = init_lua()?;

    let globals = lua.globals();
    globals.set("mv", mem_view::load_mem_view_module(&lua)?)?;
    globals.set("mu", mod_util::load_mod_util_module(&lua)?)?;

    let filepath = Path::new(APP_ROOT).join(DIR_SCRIPTS).join("main.lua");

    let script_content = fs::read_to_string(&filepath)?;
    lua.load(&script_content)
        .set_name(filepath.to_string_lossy())
        .exec()
        .map_err(|e| anyhow!("Lua 脚本执行失败:\n{e}"))?;

    Ok(())
}

pub fn parse_module(maps: &[VMArea], filter_modules: Option<&[String]>) -> Result<Vec<ModuleData>> {
    let lua = init_lua()?;
    let globals = lua.globals();

    globals.set("maps", lua.to_value(maps)?)?;

    let filter_val = match filter_modules {
        Some(list) => lua.to_value(list)?,
        None => Table(lua.create_table()?),
    };
    globals.set("filter_modules", filter_val)?;

    let filepath = Path::new(APP_ROOT)
        .join(DIR_SCRIPTS)
        .join("parse_module.lua");

    let script_content = fs::read_to_string(&filepath)?;
    let result_value: Value = lua
        .load(&script_content)
        .set_name("parse_module.lua")
        .eval()
        .map_err(|e| anyhow!("解析模块脚本错误:\n{e}"))?;

    let modules: Vec<ModuleData> = lua
        .from_value(result_value)
        .map_err(|e| anyhow!("Lua 结果反序列化失败: {e}"))?;

    Ok(modules)
}

pub fn parse_regions(maps: &[VMArea], range_type: RangeType) -> Result<Vec<VMArea>> {
    let lua = init_lua()?;
    let globals = lua.globals();

    globals.set("maps", lua.to_value(maps)?)?;

    let rt_str = match range_type {
        RangeType::All => "All",
        RangeType::Default => "Default",
    };
    globals.set("range_type", rt_str)?;

    let filepath = Path::new(APP_ROOT)
        .join(DIR_SCRIPTS)
        .join("parse_regions.lua");

    let script_content = fs::read_to_string(&filepath)?;
    let result_value: Value = lua
        .load(&script_content)
        .set_name("parse_regions.lua")
        .eval()
        .map_err(|e| anyhow!("解析区块脚本错误:\n{e}"))?;

    let regions: Vec<VMArea> = lua
        .from_value(result_value)
        .map_err(|e| anyhow!("Lua 结果反序列化失败: {e}"))?;

    Ok(regions)
}
