use crate::chainer::{ModuleData, models::Pointer, search};
use crate::proc::maps::{VMArea, is_addr_mapped};
use crate::utils::numeric;

use anyhow::{Context, Result};
use memmap2::Mmap;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::str;
use std::time::Instant;

fn get_file_len(path: &str) -> Result<usize> {
    let len = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {path}"))?
        .len() as usize;
    Ok(len)
}

fn find_valid_lines<F>(data: &[u8], predicate: F) -> Vec<&[u8]>
where
    F: Fn(&&[u8]) -> bool + Sync + Send,
{
    let clean_data = data.strip_suffix(b"\n").unwrap_or(data);
    clean_data
        .par_split(|&b| b == b'\n')
        .filter(|line| predicate(line))
        .collect()
}

fn determine_file_roles<'a>(path1: &'a str, path2: &'a str) -> Result<(&'a str, &'a str)> {
    let len1 = get_file_len(path1)?;
    let len2 = get_file_len(path2)?;

    if len1 == 0 || len2 == 0 {
        return Ok(("", ""));
    }

    if len1 < len2 {
        Ok((path1, path2))
    } else {
        Ok((path2, path1))
    }
}

fn map_file_to_memory(path: &str) -> Result<Mmap> {
    let file = File::open(path).with_context(|| format!("Failed to open {path}"))?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}

fn build_index(data: &[u8]) -> FxHashSet<&[u8]> {
    let clean_data = data.strip_suffix(b"\n").unwrap_or(data);
    clean_data.par_split(|&b| b == b'\n').collect()
}

fn handle_output(output_path: &str, matched_lines: &[&[u8]]) -> Result<()> {
    if matched_lines.is_empty() {
        fs::remove_file(output_path).ok();
        return Ok(());
    }

    let out_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file {output_path}"))?;

    let mut writer = BufWriter::new(out_file);

    for &line in matched_lines {
        writer.write_all(line)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    Ok(())
}

pub fn compare_files(path1: &str, path2: &str, output_path: &str) -> Result<usize> {
    let (small_path, big_path) = determine_file_roles(path1, path2)?;

    if small_path.is_empty() {
        return Ok(0);
    }

    let mmap_small = map_file_to_memory(small_path)?;
    let mmap_big = map_file_to_memory(big_path)?;

    let dictionary_set = build_index(&mmap_small);

    let matched_lines = find_valid_lines(&mmap_big, |&line| dictionary_set.contains(line));
    let match_count = matched_lines.len();

    handle_output(output_path, &matched_lines)?;

    Ok(match_count)
}

fn is_match(
    line: &[u8],
    module_map: &FxHashMap<&[u8], &ModuleData>,
    areas: &[VMArea],
    pointers: &[Pointer],
    address: &Option<u64>,
) -> bool {
    let Some(plus_idx) = line.iter().position(|&b| b == b'+') else {
        return false;
    };

    let module_name = &line[..plus_idx];
    let Some(module_data) = module_map.get(module_name) else {
        return false;
    };

    let module_start = module_data.start;
    let mut offsets_bytes = &line[plus_idx + 1..];

    let mut pop_offset = || -> (Option<u64>, bool) {
        let arrow_idx = offsets_bytes.windows(2).position(|w| w == b"->");

        let (offset_bytes, has_next) = if let Some(idx) = arrow_idx {
            let res = &offsets_bytes[..idx];
            offsets_bytes = &offsets_bytes[idx + 2..];
            (res, true)
        } else {
            let res = offsets_bytes;
            offsets_bytes = &[];
            (res, false)
        };

        (numeric::parse_hex_to_u64(offset_bytes), has_next)
    };

    let (first_offset_opt, mut has_next) = pop_offset();
    let Some(first_offset) = first_offset_opt else {
        return false;
    };

    let mut current_addr = module_start + first_offset;

    while has_next {
        if !is_addr_mapped(areas, current_addr) {
            return false;
        }

        let Ok(idx) = pointers.binary_search_by_key(&current_addr, |p| p.address) else {
            return false;
        };
        let base_value = pointers[idx].value;

        let (offset_opt, next_arrow) = pop_offset();
        let Some(offset) = offset_opt else {
            return false;
        };

        current_addr = base_value + offset;
        has_next = next_arrow;
    }

    address.is_none_or(|addr| addr == current_addr)
}

pub fn filter_file(
    read_path: &str,
    output_path: &str,
    areas: &[VMArea],
    modules: &[ModuleData],
    address: Option<u64>,
) -> Result<usize> {
    let len = get_file_len(read_path)?;

    if len == 0 {
        return Ok(0);
    }

    let create_timer = Instant::now();
    println!("开始创建指针集");

    let pointers = search::get_pointers(areas)?;

    let module_map: FxHashMap<&[u8], &ModuleData> =
        modules.iter().map(|m| (m.name.as_bytes(), m)).collect();

    println!(
        "创建指针集完成({})条，耗时：{:.4} 秒",
        pointers.size(),
        create_timer.elapsed().as_secs_f64()
    );

    let mmap = map_file_to_memory(read_path)?;
    let matched_lines = find_valid_lines(&mmap, |&line| {
        is_match(line, &module_map, areas, &pointers, &address)
    });
    let matches = matched_lines.len();

    handle_output(output_path, &matched_lines)?;

    Ok(matches)
}
