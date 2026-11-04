mod scan_modules;

use super::super::menu::{Menu, menu, run_menu};
use super::super::terminal::Terminal;

use crate::config::{self, StringOrVec};
use crate::errors::AppError;

use anyhow::{Result, bail};
use colored::Colorize;
use inquire::Text;

fn change_scan_addresses() -> Result<bool> {
    let mut toml = config::get_mut_global()?;
    let old_addresses = toml.get_target_config()?.addresses.clone();

    if let Ok(input) = Text::new("请输入新地址(空格分隔)：").prompt() {
        let input = input.trim();
        if input.is_empty() {
            bail!(AppError::EmptyInput);
        }

        let mut new_addresses = Vec::new();
        for s in input.split_whitespace() {
            let addr = u64::from_str_radix(s, 16).map_err(|_| AppError::InvalidHex)?;
            new_addresses.push(addr);
        }

        if !new_addresses.is_empty() {
            let process = toml.get_mut_target_config()?;
            process.addresses = new_addresses.clone();
            toml.save()?;

            Menu::show_update(
                &format!("{old_addresses:X?}"),
                &format!("{new_addresses:X?}"),
            );
            return Ok(true);
        }
    }
    Terminal::clear();
    Ok(true)
}

fn change_scan_depth() -> Result<bool> {
    let mut toml = config::get_mut_global()?;
    let old_depth = toml.get_target_config()?.depth;

    if let Ok(input) = Text::new("请输入新深度：").prompt() {
        let input = input.trim();
        if input.is_empty() {
            bail!(AppError::EmptyInput);
        }

        let d = input
            .parse::<usize>()
            .map_err(|_| AppError::InvalidPositiveInteger)?;

        Menu::show_update(&old_depth.to_string(), &d.to_string());

        toml.get_mut_target_config()?.depth = d;
        toml.save()?;
        return Ok(true);
    }
    Terminal::clear();
    Ok(true)
}

fn change_scan_range() -> Result<bool> {
    let mut toml = config::get_mut_global()?;
    let old_offset = toml.get_target_config()?.offset;

    if let Ok(input) = Text::new("请输入新范围：").prompt() {
        let input = input.trim();
        if input.is_empty() {
            bail!(AppError::EmptyInput);
        }

        let o = input
            .parse::<usize>()
            .map_err(|_| AppError::InvalidPositiveInteger)?;

        Menu::show_update(&old_offset.to_string(), &o.to_string());

        toml.get_mut_target_config()?.offset = o;
        toml.save()?;
        return Ok(true);
    }
    Terminal::clear();
    Ok(true)
}

fn reset_scan_params() -> Result<bool> {
    let mut toml = config::get_mut_global()?;
    let process = toml.get_mut_target_config()?;
    process.modules = StringOrVec::String("all".to_string());
    toml.save()?;

    Menu::show_success("扫描参数重置成功！");
    Ok(true)
}

pub fn run() -> Result<bool> {
    Terminal::clear();

    run_menu! {
        init: {
            let toml = config::get_global()?;
            let process = toml.get_target_config()?;

            println!("{}", Terminal::line());
            println!("{}{}", "扫描地址：".bright_green(), format!("{:X?}", &process.addresses).bright_magenta());
            println!("{}{}", "扫描深度：".bright_green(), process.depth.to_string().bright_magenta());
            println!("{}{}", "偏移范围：".bright_green(), process.offset.to_string().bright_magenta());
            println!("{}{}", "目标模块：".bright_green(), format!("{:?}", &process.modules.as_vec()).bright_magenta());
        },
        error: |e| {
            Menu::show_error(&e.to_string());
        },
        back(top): ["0"] "返回上一步",
        "扫描地址" => change_scan_addresses(),
        "扫描深度" => change_scan_depth(),
        "扫描范围" => change_scan_range(),
        "扫描模块" => scan_modules::run(),
        ["r"] "重置参数" => reset_scan_params(),
    }

    Terminal::clear();
    Ok(true)
}
