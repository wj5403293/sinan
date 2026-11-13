use super::menu::{Menu, menu, run_menu};
use super::terminal::Terminal;

use crate::config;
use crate::errors::AppError;
use crate::proc;

use anyhow::{Result, bail};
use colored::Colorize;
use inquire::{Select, Text};

fn change_target() -> Result<bool> {
    let mut toml = config::get_mut_global()?;

    let mut keys: Vec<String> = toml.processes.keys().cloned().collect();
    keys.push("✨ 手动输入 (PID 或 名称)".to_string());
    keys.push("❌ 取消".to_string());

    if let Ok(choice) = Select::new("请选择或输入新的索引目标：", keys)
        .without_help_message()
        .prompt()
    {
        if choice == "❌ 取消" {
            Terminal::clear();
            return Ok(true);
        }

        let new_target = if choice == "✨ 手动输入 (PID 或 名称)" {
            Text::new("请输入目标：").prompt().unwrap_or_default()
        } else {
            choice
        }
        .trim()
        .to_string();

        if new_target.is_empty() {
            bail!(AppError::EmptyInput);
        }

        proc::set_target(&new_target);

        let old_target = toml.base.target.clone();
        Menu::show_update(&format!("\"{old_target}\""), &format!("\"{new_target}\""));
        toml.set_target(new_target)?;
    }
    Ok(true)
}

fn toggle_range() -> Result<bool> {
    let mut toml = config::get_mut_global()?;

    let old_range = toml.base.range.clone();
    let new_range = if old_range == "default" {
        "all"
    } else {
        "default"
    };

    Menu::show_update(&format!("\"{old_range}\""), &format!("\"{new_range}\""));

    toml.base.range = new_range.to_string();
    toml.save()?;

    Ok(true)
}

fn toggle_mode() -> Result<bool> {
    let mut toml = config::get_mut_global()?;

    let old_mode = toml.base.mode.clone();
    let new_mode = if old_mode == "exhaustive" {
        "smart"
    } else {
        "exhaustive"
    };

    Menu::show_update(&format!("\"{old_mode}\""), &format!("\"{new_mode}\""));

    toml.base.mode = new_mode.to_string();
    toml.save()?;

    Ok(true)
}

fn reset_config() -> Result<bool> {
    let mut toml = config::get_mut_global()?;
    toml.reset()?;

    Menu::show_success("配置重置成功！");

    Ok(true)
}

pub fn run() -> Result<bool> {
    Terminal::clear();

    run_menu! {
        init: {
            println!("{}", Terminal::line());
            let toml = config::get_global()?;

            let range_str = match toml.base.range.as_str() {
                "default" => "默认".to_string(),
                "all" => "全部".to_string(),
                _ => format!("未知：{}", toml.base.range),
            };

            let mode_str = match toml.base.mode.as_str() {
                "exhaustive" => "穷举".to_string(),
                "smart" => "智能".to_string(),
                _ => format!("未知：{}", toml.base.mode),
            };

            println!("{}{}", "应用包名或进程ID：".bright_green(), format!("\"{}\"", toml.base.target).bright_magenta());
            println!("{}{}", "内存范围：".bright_green(), format!("\"{range_str}\"").bright_magenta());
            println!("{}{}", "扫描模式：".bright_green(), format!("\"{mode_str}\"").bright_magenta());
        },
        error: |e| {
            Menu::show_error(&format!("{e:#}"));
        },
        back(top): ["0"] "返回主菜单",
        "应用包名" => change_target(),
        "内存范围" => toggle_range(),
        "扫描模式" => toggle_mode(),
        ["r"] "重置配置" => reset_config(),
    }

    Terminal::clear();
    Ok(true)
}
