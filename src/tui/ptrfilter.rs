use super::menu::{Menu, menu, run_menu};
use super::terminal::Terminal;

use crate::chainer::RangeType;
use crate::config;
use crate::errors::AppError;
use crate::filters;
use crate::proc;

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Text};

fn binary_compare_menu() -> Result<bool> {
    if let Some(p1) = Menu::select_file(".bin", "请输入文件路径1：")
        && let Some(p2) = Menu::select_file(".bin", "请输入文件路径2：")
        && Confirm::new("确认继续？")
            .with_default(true)
            .prompt()
            .unwrap_or(false)
    {
        Terminal::clear();
        filters::binary_compare_to_text(&p1, &p2)?;
        Terminal::wait_enter();
        Terminal::clear();
    }
    Ok(true)
}

fn text_filter_menu() -> Result<bool> {
    let toml = config::get_global()?;
    let range = &toml.base.range;

    if let Some(p) = Menu::select_file(".txt", "请输入文件路径：") {
        let addr_str = Text::new("请输入过滤地址（回车空则为加载全部）：")
            .prompt()
            .unwrap_or_default();
        if Confirm::new("确认继续？")
            .with_default(true)
            .prompt()
            .unwrap_or(false)
        {
            Terminal::clear();
            let addr = if addr_str.trim().is_empty() {
                None
            } else {
                Some(u64::from_str_radix(addr_str.trim(), 16).map_err(|_| AppError::InvalidHex)?)
            };
            let range_type = if range == "default" {
                RangeType::Default
            } else {
                RangeType::All
            };
            filters::text_filter_to_text(&p, range_type, addr)?;
            Terminal::wait_enter();
        }
        Terminal::clear();
    }
    Ok(true)
}

fn text_compare_menu() -> Result<bool> {
    if let Some(p1) = Menu::select_file(".txt", "请输入文件路径1：")
        && let Some(p2) = Menu::select_file(".txt", "请输入文件路径2：")
        && Confirm::new("确认继续？")
            .with_default(true)
            .prompt()
            .unwrap_or(false)
    {
        Terminal::clear();
        filters::text_compare_to_text(&p1, &p2)?;
        Terminal::wait_enter();
        Terminal::clear();
    }
    Ok(true)
}

fn format_pointer_menu() -> Result<bool> {
    if let Some(path) = Menu::select_file(".bin", "请选择要格式化的指针文件：")
        && Confirm::new("确认继续？")
            .with_default(true)
            .prompt()
            .unwrap_or(false)
    {
        Terminal::clear();
        filters::binary_format_to_text(&path)?;
        Terminal::wait_enter();
        Terminal::clear();
    }
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

            println!(
                "\n{}\n",
                Terminal::center(&format!("当前使用配置：\"{target}\"").bright_green().to_string())
            );
        },
        error: |e| {
            Menu::show_error(&format!("{e:#}"));
        },
        back(top): ["0"] "返回主菜单",
        "二进制对比" => binary_compare_menu(),
        "文本过滤" => text_filter_menu(),
        "文本对比" => text_compare_menu(),
        "格式化指针" => format_pointer_menu(),
    }

    Terminal::clear();
    Ok(true)
}
