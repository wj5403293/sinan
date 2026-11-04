use super::super::super::menu::Menu;

use crate::config::{self, AppConfig, StringOrVec};
use crate::proc::{ensure_attached, maps::Maps};
use crate::script;

use anyhow::Result;
use colored::Colorize;
use inquire::MultiSelect;
use std::fmt;

#[derive(Clone)]
struct ModuleOption {
    name: String,
    start: u64,
    end: u64,
    display_string: String,
}

impl fmt::Display for ModuleOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_string)
    }
}

fn fetch_module_options() -> Result<Vec<ModuleOption>> {
    let (pid, _) = ensure_attached()?;
    let areas = Maps::parse(pid)?;

    let modules = script::parse_module(&areas, None)?
        .into_iter()
        .map(|m| {
            let display = format!(
                "{} {}{}{}",
                m.name.bright_red(),
                format!("{:X}", m.start).bright_green(),
                "-".bright_white(),
                format!("{:X}", m.end).bright_green()
            );
            ModuleOption {
                name: m.name,
                start: m.start,
                end: m.end,
                display_string: display,
            }
        })
        .collect();

    Ok(modules)
}

fn calculate_default_indices(options: &[ModuleOption], current_modules: &[String]) -> Vec<usize> {
    options
        .iter()
        .enumerate()
        .filter(|(_, opt)| current_modules.contains(&opt.name))
        .map(|(i, _)| i)
        .collect()
}

fn module_scorer(filter: &str, option: &ModuleOption, _: &str, _: usize) -> Option<i64> {
    let filter = filter.trim().to_lowercase();
    if filter.is_empty() {
        return Some(10);
    }

    if option.name.to_lowercase().contains(&filter) {
        return Some(2);
    }

    let clean_filter = filter.trim_start_matches("0x");

    if let Ok(input_addr) = u64::from_str_radix(clean_filter, 16)
        && input_addr >= option.start
        && input_addr <= option.end
    {
        return Some(0);
    }

    let start_hex = format!("{:x}", option.start);
    let end_hex = format!("{:x}", option.end);
    if start_hex.contains(clean_filter) || end_hex.contains(clean_filter) {
        return Some(1);
    }

    None
}

fn prompt_module_selection(
    options: Vec<ModuleOption>,
    default_indices: Vec<usize>,
) -> Result<Vec<String>, inquire::InquireError> {
    let ans = MultiSelect::new("请选择目标模块：", options)
        .with_help_message("输入文字过滤，空格选中，回车确认")
        .with_default(&default_indices)
        .with_scorer(&module_scorer)
        .with_page_size(16)
        .prompt()?;

    Ok(ans.into_iter().map(|opt| opt.name).collect())
}

fn save_config(config: &mut AppConfig, selected_names: Vec<String>) -> Result<()> {
    let target_modules = if selected_names.is_empty() {
        Menu::show_error("未选择任何模块，默认使用全量 [all]");
        StringOrVec::String("all".to_string())
    } else {
        Menu::show_success("模块更新成功！");
        StringOrVec::Vec(selected_names)
    };

    let process_config = config.get_mut_target_config()?;
    process_config.modules = target_modules;
    config.save()?;
    Ok(())
}

pub fn run() -> Result<bool> {
    let mut config = config::get_mut_global()?;

    let options = fetch_module_options()?;

    let current_modules = config
        .processes
        .get(&config.base.target)
        .map(|p| p.modules.as_vec())
        .unwrap_or_default();

    let default_indices = calculate_default_indices(&options, &current_modules);

    let selected_names = prompt_module_selection(options, default_indices)?;
    save_config(&mut config, selected_names)?;

    Ok(true)
}
