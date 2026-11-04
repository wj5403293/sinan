pub mod format;
pub mod models;
pub mod search;

mod scan;

use crate::constants::{APP_ROOT, DIR_OUT};
use crate::proc::{self, maps::Maps};
use crate::script;
use crate::utils;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeType {
    All,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanMode {
    Exhaustive,             // 穷举模式
    Smart { depth: usize }, // 智能模式（带去重）
}

#[derive(Debug, Clone)]
pub struct ScanParams {
    pub addresses: Vec<u64>,
    pub modules: Vec<String>,
    pub range_type: RangeType,
    pub scan_mode: ScanMode,
    pub depth: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleData {
    pub start: u64,
    pub end: u64,
    pub name: String,
}

pub fn scan(params: &ScanParams) -> Result<()> {
    let process_name = proc::get_target_name().context("Target Name is None")?;
    let path_buf = Path::new(APP_ROOT).join(DIR_OUT).join(&process_name);

    let filepath = utils::get_incremented_filename(
        path_buf.to_str().context("输出路径包含非UTF-8字符")?,
        ".bin",
    );

    let timer = Instant::now();

    let (pid, _) = proc::ensure_attached()?;
    println!("pid: {pid}");

    let maps = Maps::parse(pid)?;
    let areas = script::parse_regions(&maps, params.range_type)?;
    let modules = script::parse_module(&maps, Some(&params.modules))?;

    let chains = scan::start(&areas, &modules, params)?;

    println!(
        "扫描过程结束，耗时：{:.4} 秒",
        timer.elapsed().as_secs_f64()
    );
    println!("开始写入文件: {filepath}");
    format::write_chains_to_binary(&chains, &filepath)?;

    println!("写入完成，总耗时：{:.4} 秒", timer.elapsed().as_secs_f64());
    Ok(())
}

pub fn scan_relation(params: &ScanParams) -> Result<()> {
    let process_name = proc::get_target_name().context("Target Name is None")?;
    let path_buf = Path::new(APP_ROOT).join(DIR_OUT).join(&process_name);

    let filepath = utils::get_incremented_filename(
        path_buf.to_str().context("输出路径包含非UTF-8字符")?,
        ".bin",
    );
    let timer = Instant::now();

    let (pid, _) = proc::ensure_attached()?;
    println!("pid: {pid}");

    let maps = Maps::parse(pid)?;
    let areas = script::parse_regions(&maps, params.range_type)?;

    let parent_addr = params
        .addresses
        .get(1)
        .copied()
        .context("Failed to get parent address at index 1")?;

    let modules = vec![ModuleData {
        start: parent_addr,
        end: parent_addr + params.offset as u64,
        name: "父地址".to_string(),
    }];

    let mut local_params = params.clone();
    if local_params.addresses.len() > 1 {
        local_params.addresses.pop();
    }

    let chains = scan::start(&areas, &modules, &local_params)?;

    println!(
        "扫描过程结束，耗时：{:.4} 秒",
        timer.elapsed().as_secs_f64()
    );
    println!("开始写入文件: {filepath}");
    format::write_chains_to_binary(&chains, &filepath)?;
    println!("写入完成，总耗时：{:.4} 秒", timer.elapsed().as_secs_f64());
    Ok(())
}
