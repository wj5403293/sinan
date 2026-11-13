mod config;

use super::menu::{Menu, menu, run_menu};
use super::terminal::Terminal;

use crate::chainer::{self, RangeType, ScanMode, ScanParams};
use crate::config as app_config;
use crate::errors::AppError;
use crate::proc;

use anyhow::{Result, bail};
use colored::Colorize;
use inquire::{Confirm, Text};

fn basic_scan_menu() -> Result<bool> {
    let toml = app_config::get_global()?;
    let process = toml.get_target_config()?;
    let range = &toml.base.range;
    let mode = &toml.base.mode;

    let scan_mode = match mode.as_str() {
        "exhaustive" => ScanMode::Exhaustive,
        "smart" => {
            let input = Text::new("请输入限制层数（直接回车默认全部）：")
                .prompt()
                .unwrap_or_default();
            let input = input.trim();

            let depth_limit = if input.is_empty() {
                process.depth
            } else {
                input
                    .parse::<usize>()
                    .map_err(|_| AppError::InvalidPositiveInteger)?
            };
            ScanMode::Smart { depth: depth_limit }
        }
        _ => ScanMode::Exhaustive,
    };

    if Confirm::new("确认继续？")
        .with_default(true)
        .prompt()
        .unwrap_or(false)
    {
        Terminal::clear();

        let range_type = if range == "default" {
            RangeType::Default
        } else {
            RangeType::All
        };

        chainer::scan(&ScanParams {
            addresses: process.addresses.clone(),
            modules: process.modules.as_vec(),
            range_type,
            scan_mode,
            depth: process.depth,
            offset: process.offset,
        })?;

        Terminal::wait_enter();
    }
    Terminal::clear();
    Ok(true)
}

fn pointer_relation_menu() -> Result<bool> {
    let toml = app_config::get_global()?;
    let process = toml.get_target_config()?;
    let range = &toml.base.range;

    let p_addr_str = Text::new("请输入父地址：").prompt().unwrap_or_default();
    if p_addr_str.trim().is_empty() {
        bail!(AppError::EmptyInput);
    }
    let parent = u64::from_str_radix(p_addr_str.trim(), 16).map_err(|_| AppError::InvalidHex)?;

    let c_addr_str = Text::new("请输入子地址：").prompt().unwrap_or_default();
    if c_addr_str.trim().is_empty() {
        bail!(AppError::EmptyInput);
    }
    let child = u64::from_str_radix(c_addr_str.trim(), 16).map_err(|_| AppError::InvalidHex)?;

    if Confirm::new("确认继续？")
        .with_default(true)
        .prompt()
        .unwrap_or(false)
    {
        Terminal::clear();
        let range_type = if range == "default" {
            RangeType::Default
        } else {
            RangeType::All
        };
        chainer::scan_relation(&ScanParams {
            addresses: vec![child, parent],
            modules: process.modules.as_vec(),
            range_type,
            scan_mode: ScanMode::Exhaustive,
            depth: process.depth,
            offset: process.offset,
        })?;
        Terminal::wait_enter();
    }
    Terminal::clear();
    Ok(true)
}

pub fn run() -> Result<bool> {
    Terminal::clear();

    run_menu! {
        init: {
            let toml = app_config::get_global()?;
            let target = &toml.base.target;

            proc::set_target(target);
            proc::ensure_attached()?;

            println!("{}", Terminal::line());
            println!(
                "\n{}\n\n{}",
                Terminal::center(&format!("当前使用配置：\"{target}\"").bright_green().to_string()),
                Terminal::line()
            );

            let process = toml.get_target_config()?;
            println!("{}{}", "扫描地址：".bright_green(), format!("{:X?}", process.addresses).bright_magenta());
            println!("{}{}", "扫描深度：".bright_green(), process.depth.to_string().bright_magenta());
            println!("{}{}", "偏移范围：".bright_green(), process.offset.to_string().bright_magenta());
            println!("{}{}", "目标模块：".bright_green(), format!("{:?}", process.modules.as_vec()).bright_magenta());
        },
        error: |e| {
            Menu::show_error(&format!("{e:#}"));
        },
        back(top): ["0"] "返回主菜单",
        "扫描配置" => config::run(),
        "基础扫描" => basic_scan_menu(),
        "指针关系" => pointer_relation_menu(),
    }

    Terminal::clear();
    Ok(true)
}
