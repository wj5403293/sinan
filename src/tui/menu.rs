use super::terminal::Terminal;

use crate::constants::{APP_ROOT, DIR_OUT};

use colored::Colorize;
use inquire::{InquireError, Select};
use std::fs;
use std::path::Path;

#[derive(Clone)]
struct MenuChoice {
    id: usize,
    display: String,
    raw_desc: String,
}

impl std::fmt::Display for MenuChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

#[derive(Default)]
pub struct Menu {
    choices: Vec<MenuChoice>,
    handlers: Vec<Box<dyn Fn() -> bool>>,
    index_count: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_index_option<F>(&mut self, desc: &str, handler: F)
    where
        F: Fn() -> bool + 'static,
    {
        let id = self.handlers.len();
        self.index_count += 1;

        let formatted_desc = format!(
            "{} {}",
            format!("[{count}]", count = self.index_count).bright_yellow(),
            desc.bright_cyan()
        );
        let raw_desc = format!("[{count}] {desc}", count = self.index_count);

        self.choices.push(MenuChoice {
            id,
            display: formatted_desc,
            raw_desc,
        });
        self.handlers.push(Box::new(handler));
    }

    pub fn add_key_option<F>(&mut self, key: &str, desc: &str, handler: F)
    where
        F: Fn() -> bool + 'static,
    {
        let id = self.handlers.len();

        let formatted_desc = format!(
            "{} {}",
            format!("[{key}]").bright_yellow(),
            desc.bright_green()
        );
        let raw_desc = format!("[{key}] {desc}");

        self.choices.push(MenuChoice {
            id,
            display: formatted_desc,
            raw_desc,
        });
        self.handlers.push(Box::new(handler));
    }

    pub fn show(&self) -> bool {
        println!("{}", Terminal::line());

        let ans = Select::new(
            "请使用 ↑/↓ 选择操作并按回车 (支持打字过滤):",
            self.choices.clone(),
        )
        .with_page_size(12)
        .with_scorer(&|input: &str, option: &MenuChoice, _, _| -> Option<i64> {
            let input_lower = input.to_lowercase();
            let raw_lower = option.raw_desc.to_lowercase();

            if raw_lower.contains(&input_lower) {
                Some(0)
            } else {
                None
            }
        })
        .prompt();

        match ans {
            Ok(choice) => self.handlers[choice.id](),
            Err(InquireError::OperationInterrupted | InquireError::OperationCanceled) => {
                Self::show_cancel();
                false
            }
            Err(other) => {
                Self::show_error(&format!("发生未知错误：{other:?}"));
                true
            }
        }
    }

    pub fn show_cancel() {
        Terminal::clear();
        println!(
            "{}\n\n{}\n",
            Terminal::line(),
            Terminal::center("取消操作！").bright_green()
        );
    }

    pub fn show_update(old: &str, new: &str) {
        Terminal::clear();
        let msg = format!("{old} -> {new}");
        println!(
            "{}\n\n{}\n",
            Terminal::line(),
            Terminal::center(&msg).bright_green()
        );
    }

    pub fn show_success(text: &str) {
        Terminal::clear();
        println!(
            "{}\n\n{}\n",
            Terminal::line(),
            Terminal::center(text).bright_green()
        );
    }

    pub fn show_error(text: &str) {
        Terminal::clear();
        println!("{}\n\n{}\n", Terminal::line(), Terminal::center(text).red());
    }

    pub fn select_file(ext: &str, prompt: &str) -> Option<String> {
        let mut files_with_time = vec![];
        let target_dir = Path::new(APP_ROOT).join(DIR_OUT);

        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().and_then(|e| e.to_str())
                        == Some(ext.trim_start_matches('.'))
                {
                    let modified = path
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);

                    files_with_time.push((path.display().to_string(), modified));
                }
            }
        }

        if files_with_time.is_empty() {
            Self::show_error("目录中未找到相关文件！");
            return None;
        }

        files_with_time.sort_by_key(|b| std::cmp::Reverse(b.1));

        let files: Vec<String> = files_with_time.into_iter().map(|(path, _)| path).collect();

        let cancel_str = "❌ 取消操作！".to_string();
        let mut opts = files.clone();
        opts.push(cancel_str.clone());

        match Select::new(prompt, opts).without_help_message().prompt() {
            Ok(choice) if choice == cancel_str => {
                Self::show_cancel();
                None
            }
            Ok(choice) => Some(choice),
            Err(_) => {
                Self::show_cancel();
                None
            }
        }
    }
}

macro_rules! menu {
    (
        error: |$err:ident| $err_handler:block,
        back($pos:ident): [$back_key:expr] $back_desc:literal,
        $( $( [$key:expr] )? $desc:literal => $handler:expr ),* $(,)?
    ) => {{
        let mut m = Menu::new();
        let back_handler = || false;

        #[allow(unused_assignments)]
        if stringify!($pos) == "top" {
            m.add_key_option($back_key, $back_desc, back_handler);
        }

        $(
            let wrapper = || -> bool {
                #[allow(clippy::redundant_closure_call)]
                let res = (|| -> Result<bool, anyhow::Error> { $handler })();

                match res {
                    Ok(b) => b,
                    Err($err) => {
                        $err_handler;
                        true
                    }
                }
            };
            menu!(@add_option m, $desc, wrapper, $([$key] )? );
        )*

        if stringify!($pos) == "bottom" {
            m.add_key_option($back_key, $back_desc, back_handler);
        }

        m
    }};

    (@add_option $m:ident, $desc:expr, $handler:expr, ) => { $m.add_index_option($desc, $handler); };
    (@add_option $m:ident, $desc:expr, $handler:expr, [$key:expr]) => { $m.add_key_option($key, $desc, $handler); };
}

macro_rules! run_menu {
    (
        $( init: $init:block, )?
        error: |$err:ident| $err_handler:block,
        back($pos:ident): [$back_key:expr] $back_desc:literal,
        $( $( [$key:expr] )? $desc:literal => $handler:expr ),* $(,)?
    ) => {
        loop {
            $( $init )?
            let m = menu! {
                error: |$err| $err_handler,
                back($pos): [$back_key] $back_desc,
                $( $( [$key] )? $desc => $handler ),*
            };
            if !m.show() {
                break;
            }
        }
    };
}

pub(crate) use menu;
pub(crate) use run_menu;
