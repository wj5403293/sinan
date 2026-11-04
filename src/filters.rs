mod binary;
mod text;

use crate::chainer::RangeType;
use crate::constants::{APP_ROOT, DIR_OUT};
use crate::proc::{self, maps::Maps};
use crate::script;
use crate::utils;

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

pub fn text_compare_to_text(filepath1: &str, filepath2: &str) -> Result<()> {
    println!("开始文本对比");

    let timer = Instant::now();

    let basename1 = utils::basename(filepath1);
    let basename2 = utils::basename(filepath2);
    let new_name = utils::get_incremented_filename(
        Path::new(APP_ROOT)
            .join(DIR_OUT)
            .join(format!("{basename1}&{basename2}"))
            .to_str()
            .context("路径包含非UTF-8字符")?,
        ".txt",
    );
    let count = text::compare_files(filepath1, filepath2, &new_name)?;

    println!("文本对比完成，有效数量: {count}");
    println!("输出: {new_name}");
    println!("写入完成，总耗时：{:.4} 秒", timer.elapsed().as_secs_f64());
    Ok(())
}

pub fn binary_compare_to_text(filepath1: &str, filepath2: &str) -> Result<()> {
    println!("开始二进制对比");

    let timer = Instant::now();

    let basename1 = utils::basename(filepath1);
    let basename2 = utils::basename(filepath2);
    let new_name = utils::get_incremented_filename(
        Path::new(APP_ROOT)
            .join(DIR_OUT)
            .join(format!("{basename1}&{basename2}"))
            .to_str()
            .context("路径包含非UTF-8字符")?,
        ".txt",
    );
    let count = binary::compare_files(filepath1, filepath2, &new_name)?;

    println!("二进制对比完成，有效数量: {count}");
    println!("输出: {new_name}");
    println!("写入完成，总耗时：{:.4} 秒", timer.elapsed().as_secs_f64());

    Ok(())
}

pub fn text_filter_to_text(filepath: &str, range_type: RangeType, addr: Option<u64>) -> Result<()> {
    println!("开始文本过滤");

    let basename = utils::basename(filepath);
    let new_name = utils::get_incremented_filename(
        Path::new(APP_ROOT)
            .join(DIR_OUT)
            .join(basename)
            .to_str()
            .context("路径包含非UTF-8字符")?,
        ".txt",
    );

    let timer = Instant::now();

    let (pid, _) = proc::ensure_attached()?;
    println!("pid: {pid}");

    let maps = Maps::parse(pid)?;
    let areas = script::parse_regions(&maps, range_type)?;
    let modules = script::parse_module(&maps, None)?;

    let count = text::filter_file(filepath, &new_name, &areas, &modules, addr)?;

    println!("文本过滤完成，有效数量: {count}");
    println!("输出: {new_name}");
    println!("写入完成，总耗时：{:.4} 秒", timer.elapsed().as_secs_f64());

    Ok(())
}

pub fn binary_format_to_text(filepath: &str) -> Result<()> {
    println!("开始格式化");
    let timer = Instant::now();

    let basename = utils::basename(filepath);
    let new_name = utils::get_incremented_filename(
        Path::new(APP_ROOT)
            .join(DIR_OUT)
            .join(basename)
            .to_str()
            .context("路径包含非UTF-8字符")?,
        ".txt",
    );

    let count = binary::format_file(filepath, &new_name)?;

    println!("格式化完成，总数量: {count}");
    println!("输出: {new_name}");
    println!("写入完成，总耗时：{:.4} 秒", timer.elapsed().as_secs_f64());
    Ok(())
}
