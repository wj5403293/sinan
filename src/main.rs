#[cfg(not(target_pointer_width = "64"))]
compile_error!("This crate supports 64-bit systems only");

pub mod chainer;
pub mod config;
pub mod constants;
pub mod errors;
pub mod filters;
pub mod proc;
pub mod script;
pub mod tui;
pub mod utils;

use crate::constants::{APP_ROOT, DIR_OUT, DIR_SCRIPTS, DIR_TMP};

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;
use tui::terminal::Terminal;

fn init_fs() -> Result<bool> {
    let root = Path::new(APP_ROOT);
    fs::create_dir_all(root.join(DIR_OUT))?;
    fs::create_dir_all(root.join(DIR_TMP))?;
    fs::create_dir_all(root.join(DIR_SCRIPTS))?;
    config::init()?;

    Ok(true)
}

#[cfg(unix)]
fn check_and_switch_root() -> &'static str {
    use {
        std::env,
        std::os::unix::process::CommandExt,
        std::process::{self, Command},
    };

    let mut mode = "ROOT";

    let is_root = unsafe { libc::geteuid() } == 0;

    if !is_root {
        let path = match env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("readlink (current_exe) failed: {e}");
                process::exit(1);
            }
        };

        let _ = Command::new("su").arg("-c").arg(&path).exec();

        mode = "NON-ROOT";
    }

    mode
}

#[cfg(not(unix))]
pub fn check_and_switch_root() -> &'static str {
    "unknown"
}

fn main() {
    Terminal::clear();

    if let Err(e) = init_fs() {
        println!("{e:?}");
        return;
    }

    let mode = check_and_switch_root();

    println!("{}", Terminal::line());
    println!(
        "{}",
        Terminal::center(&format!("当前模式：{mode}")).bright_magenta()
    );

    tui::run();

    Terminal::clear();
    println!("{}", Terminal::center("感谢使用，再见！").green());
}
