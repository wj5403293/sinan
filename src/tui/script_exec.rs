use super::menu::{Menu, menu, run_menu};
use super::terminal::Terminal;

use crate::config;
use crate::proc;
use crate::script;

use anyhow::Result;
use colored::Colorize;

fn execute_script_menu() -> Result<bool> {
    Terminal::clear();

    if let Err(err) = script::lua_start() {
        println!("{err}");
    }

    Terminal::wait_enter();
    Terminal::clear();

    Ok(true)
}

pub fn run() -> Result<bool> {
    Terminal::clear();

    run_menu! {
        init: {
            println!("{}", Terminal::line());

            let toml = config::get_global()?;
            let target = &toml.base.target;

            proc::set_target(target);
            proc::ensure_attached()?;

            println!(
                "\n{}\n",
                Terminal::center(&format!("当前设置目标：\"{target}\"").bright_green().to_string())
            );
        },
        error: |e| {
            Menu::show_error(&e.to_string());
        },
        back(top): ["0"] "返回主菜单",
        "执行脚本" => execute_script_menu(),
    }

    Terminal::clear();
    Ok(true)
}
