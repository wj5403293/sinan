pub mod terminal;

mod config;
mod menu;
mod ptrfilter;
mod ptrscan;
mod script_exec;

use crate::constants::VERSION;

use anyhow::Result;
use colored::{Color, Colorize};
use menu::{Menu, menu, run_menu};
use terminal::Terminal;

fn help() -> Result<bool> {
    let _app_name = "司南";

    let ai_credits = ["DeepSeek", "ChatGPT", "Gemini", "Claude", "豆包"];

    let disclaimer =
        r"1. 本工具/内容/服务仅限用于技术交流、学术研究及个人学习用途，不构成任何实际操作建议。
2. 使用者需自行承担所有风险，包括但不限于因使用导致的直接或间接损失。
3. 本工具提供的信息未经过全面验证，开发者不承担因信息错误导致的责任。
4. 禁止将本工具用于违反法律法规或侵害他人权益的行为。
5. 涉及第三方资源时，安全性与合法性由第三方负责。"
            .to_string();

    Terminal::clear();
    println!("{}", Terminal::line());

    println!("{}", Terminal::center(&format!("版本：{VERSION}")));
    println!("{}\n", Terminal::center("作者：🐟"));

    println!("{}", Terminal::center("致谢 (排名不分先后)"));
    for (i, ai) in ai_credits.iter().enumerate() {
        println!("{}", Terminal::center(&format!("{}. {}", i + 1, ai)));
    }
    println!();

    println!("{}", Terminal::center("免责声明"));
    for line in disclaimer.trim().lines() {
        println!("{line}");
    }
    println!("{}", Terminal::line());

    Terminal::wait_enter();

    Terminal::clear();
    Ok(true)
}

fn art_print() {
    println!("{}", Terminal::line());

    let banner_art = r" ######   ####    ##    ##     ###     ##    ##
##    ##   ##     ###   ##    ## ##    ###   ##
##         ##     ####  ##   ##   ##   ####  ##
 ######    ##     ## ## ##  ##     ##  ## ## ##
      ##   ##     ##  ####  #########  ##  ####
##    ##   ##     ##   ###  ##     ##  ##   ###
 ######   ####    ##    ##  ##     ##  ##    ##"
        .to_string();

    let banner_colors = [
        Color::BrightRed,
        Color::BrightGreen,
        Color::BrightYellow,
        Color::BrightBlue,
        Color::BrightMagenta,
        Color::BrightCyan,
        Color::BrightBlack,
    ];
    for (i, line) in banner_art.trim().lines().enumerate() {
        println!("{}", Terminal::center(line).color(banner_colors[i]));
    }
}

pub fn run() {
    run_menu! {
        init: {
            art_print();
        },
        error: |e| {
            Menu::show_error(&format!("{e:#}"));
        },
        back(bottom): ["q"] "退出程序",
        "基础配置"       => config::run(),
        "扫描指针"       => ptrscan::run(),
        "指针过滤"       => ptrfilter::run(),
        "模拟脚本"       => script_exec::run(),
        ["h"] "使用须知" => help(),
    }
}
