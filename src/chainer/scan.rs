use super::models::{CoreData, Edge, Module, ModuleArea, Pointer};
use super::{ModuleData, ScanMode, ScanParams, format, search};

use crate::proc::maps::VMArea;
use crate::utils::map_vec::MapVec;

use anyhow::{Result, bail};
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

type VecPtrs = Vec<MapVec<Pointer>>;

#[inline(always)]
fn pointer_in_range(targets: &[Pointer], target: u64, offset: usize) -> bool {
    let start_idx = targets.partition_point(|t| t.address < target);

    start_idx < targets.len() && targets[start_idx].address <= target + offset as u64
}

#[inline(always)]
fn find_pointed_to_range(ptrs: &[Pointer], start: u64, end: u64) -> Option<(usize, usize)> {
    let first = ptrs.partition_point(|p| p.address < start);

    if first == ptrs.len() || ptrs[first].address > end {
        return None;
    }

    let last = ptrs.partition_point(|p| p.address <= end) - 1;

    Some((first, last))
}

#[inline(always)]
fn find_pointer_edge(ptrs: &[Pointer], ptr: &Pointer, offset: usize) -> Option<Edge> {
    let (first, last) = find_pointed_to_range(ptrs, ptr.value, ptr.value + offset as u64)?;

    Some(Edge {
        from: ptr.address,
        to: ptr.value,
        start: first as u32,
        size: (last - first + 1) as u32,
    })
}

fn scan_pointer_chains(
    pointers: &MapVec<Pointer>,
    modules: &[ModuleData],
    params: &ScanParams,
) -> Result<(VecPtrs, VecPtrs)> {
    let mut mem_vecs = MapVec::<Pointer>::news(params.depth + 1)?;
    let mut mod_vecs = MapVec::<Pointer>::news(params.depth)?;

    for &addr in &params.addresses {
        mem_vecs[0].write_single(&Pointer {
            address: addr,
            value: addr,
        })?;
    }
    mem_vecs[0].map()?;

    let mut filter = FxHashSet::<u64>::default();

    let mut count = 0;
    for level in 0..params.depth {
        let current_targets = &mem_vecs[level];

        let temps: Vec<Vec<Pointer>> = pointers
            .par_chunks(1024 * 64)
            .map(|chunk| {
                chunk
                    .iter()
                    .filter(|ptr| pointer_in_range(current_targets, ptr.value, params.offset))
                    .copied()
                    .collect::<Vec<Pointer>>()
            })
            .collect();

        match params.scan_mode {
            ScanMode::Exhaustive => {
                for temp in temps {
                    mem_vecs[level + 1].write_slice(&temp)?;
                }
            }
            ScanMode::Smart { depth } => {
                if level < depth {
                    for dat in temps.iter().flatten() {
                        if filter.insert(dat.address) {
                            mem_vecs[level + 1].write_single(dat)?;
                        }
                    }
                } else {
                    for temp in temps {
                        mem_vecs[level + 1].write_slice(&temp)?;
                    }
                }
            }
        }

        if mem_vecs[level + 1].is_empty() {
            break;
        }
        mem_vecs[level + 1].map()?;

        let next_level_ptrs = &mem_vecs[level + 1];
        for module in modules {
            if let Some((first, last)) =
                find_pointed_to_range(next_level_ptrs, module.start, module.end)
            {
                mod_vecs[level].write_slice(&next_level_ptrs[first..=last])?;
            }
        }

        print!(
            "层数: {}, 指针数量: {}",
            level + 1,
            mem_vecs[level + 1].size()
        );

        if !mod_vecs[level].is_empty() {
            mod_vecs[level].map()?;
            print!("，基址数量: {}", mod_vecs[level].size());
            count += mod_vecs[level].size();
        }
        println!();
    }

    if count == 0 {
        bail!("扫描失败，无有效基址");
    }

    Ok((mem_vecs, mod_vecs))
}

fn build_pointer_chains(
    mem_vecs: &[MapVec<Pointer>],
    mod_vecs: &[MapVec<Pointer>],
    params: &ScanParams,
) -> Result<VecPtrs> {
    let mut numbers: Vec<Vec<usize>> = vec![Vec::new(); params.depth + 1];

    for level in (0..params.depth).rev() {
        let pointers_size = mem_vecs[level].size();
        if pointers_size == 0 {
            continue;
        }

        let to_insert: Vec<AtomicBool> =
            (0..pointers_size).map(|_| AtomicBool::new(false)).collect();

        let mark_pointers = |ptr: &Pointer| {
            if let Some((first, last)) = find_pointed_to_range(
                &mem_vecs[level],
                ptr.value,
                ptr.value + params.offset as u64,
            ) {
                for item in &to_insert[first..=last] {
                    item.store(true, Ordering::Relaxed);
                }
            }
        };

        mod_vecs[level][..].par_iter().for_each(mark_pointers);
        if !numbers[level + 1].is_empty() {
            let next_level_mem = &mem_vecs[level + 1];
            numbers[level + 1].par_iter().for_each(|&idx| {
                mark_pointers(&next_level_mem[idx]);
            });
        }

        numbers[level] = to_insert
            .into_par_iter()
            .enumerate()
            .filter_map(|(i, b)| if b.into_inner() { Some(i) } else { None })
            .collect();
    }

    let mut pointer_vecs = MapVec::<Pointer>::news(params.depth)?;
    for level in (0..params.depth).rev() {
        if numbers[level].is_empty() {
            continue;
        }

        for &idx in &numbers[level] {
            pointer_vecs[level].write_single(&mem_vecs[level][idx])?;
        }
        pointer_vecs[level].map()?;
    }

    Ok(pointer_vecs)
}

fn filter_valid_modules(
    mod_vecs: &[MapVec<Pointer>],
    mod_result: &[ModuleData],
    depth: usize,
) -> (Vec<Module>, Vec<ModuleData>) {
    let mut modules = Vec::new();
    let mut local_result = Vec::new();

    for mod_data in mod_result {
        let is_valid = (0..depth).rev().any(|level| {
            mod_vecs[level]
                .iter()
                .any(|ptr| ptr.address >= mod_data.start && ptr.address <= mod_data.end)
        });

        if is_valid {
            modules.push(Module {
                start: mod_data.start,
                name: mod_data.name.clone(),
            });
            local_result.push(mod_data.clone());
        }
    }

    (modules, local_result)
}

fn build_module_areas(
    mod_result: &[ModuleData],
    mod_vecs: &[MapVec<Pointer>],
    pointer_vecs: &[MapVec<Pointer>],
    params: &ScanParams,
) -> Vec<ModuleArea> {
    let mut areas = Vec::new();

    for level in 0..params.depth {
        if mod_vecs[level].is_empty() {
            continue;
        }

        let pointers_slice = &pointer_vecs[level];
        for (i, module) in mod_result.iter().enumerate() {
            let edges: Vec<Edge> = mod_vecs[level]
                .par_iter()
                .filter(|ptr| ptr.address >= module.start && ptr.address <= module.end)
                .filter_map(|ptr| find_pointer_edge(pointers_slice, ptr, params.offset))
                .collect();

            if edges.is_empty() {
                continue;
            }

            println!("第 {} 层 {} found: {}", level + 1, module.name, edges.len());

            areas.push(ModuleArea {
                level: level + 1,
                index: i,
                edges,
            });
        }
    }

    areas
}

fn build_pointer_tree(pointer_vecs: &[MapVec<Pointer>], params: &ScanParams) -> Vec<Vec<Edge>> {
    let mut tree = Vec::with_capacity(params.depth);

    let first_level = &pointer_vecs[0];
    let edges: Vec<Edge> = first_level
        .iter()
        .map(|ptr| Edge {
            from: ptr.address,
            to: ptr.value,
            start: 0,
            size: 1,
        })
        .collect();
    tree.push(edges);

    for level in 1..params.depth {
        let previous_level = &pointer_vecs[level - 1];
        let current_level = &pointer_vecs[level];

        let edges: Vec<Edge> = current_level
            .par_iter()
            .filter_map(|ptr| find_pointer_edge(previous_level, ptr, params.offset))
            .collect();

        tree.push(edges);
    }

    tree
}

pub fn start(areas: &[VMArea], modules: &[ModuleData], params: &ScanParams) -> Result<CoreData> {
    let create_timer = Instant::now();
    println!("开始创建指针集");

    let pointers = search::get_pointers(areas)?;
    if pointers.is_empty() {
        bail!("获取指针失败, 指针集为空");
    }

    println!(
        "创建指针集完成({})条，耗时：{:.4} 秒",
        pointers.size(),
        create_timer.elapsed().as_secs_f64()
    );

    let scan_timer = Instant::now();
    println!("开始扫描");

    let (mem_vecs, mod_vecs) = scan_pointer_chains(&pointers, modules, params)?;

    println!(
        "扫描完成，耗时：{:.4} 秒",
        scan_timer.elapsed().as_secs_f64()
    );

    let build_chains_timer = Instant::now();
    println!("开始构建指针链");

    let pointer_vecs = build_pointer_chains(&mem_vecs, &mod_vecs, params)?;

    println!(
        "构建指针链完成，耗时：{:.4} 秒",
        build_chains_timer.elapsed().as_secs_f64()
    );

    let find_mofule_timer = Instant::now();
    println!("开始查找模块指针");

    let (valid_modules, read_modules) = filter_valid_modules(&mod_vecs, modules, params.depth);

    let mareas = build_module_areas(&read_modules, &mod_vecs, &pointer_vecs, params);

    println!(
        "查找模块指针完成，耗时：{:.4} 秒",
        find_mofule_timer.elapsed().as_secs_f64()
    );

    let build_tree_timer = Instant::now();
    println!("开始构建指针树");

    let tree = build_pointer_tree(&pointer_vecs, params);

    println!(
        "构建指针树完成，耗时：{:.4} 秒",
        build_tree_timer.elapsed().as_secs_f64()
    );

    let count = format::calculate_pointer_chain_count(&mareas, &tree);

    println!("指针链数量: {count}");

    Ok(CoreData {
        areas: mareas,
        modules: valid_modules,
        tree,
    })
}
