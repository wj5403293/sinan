use crate::chainer::{
    format,
    models::{CoreData, Edge, Module, ModuleArea},
};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

fn find_intersections<F>(
    edges: (&[Edge], &[Edge]),
    bases: (u64, u64),
    mut on_match: F,
) -> Result<()>
where
    F: FnMut(&Edge, &Edge, u32, usize) -> Result<()>,
{
    let (edges_s, edges_l) = edges;
    let (base_s, base_l) = bases;

    let (mut i, mut j) = (0, 0);
    while i < edges_s.len() && j < edges_l.len() {
        let offset_s = (edges_s[i].from - base_s) as u32;
        let offset_l = (edges_l[j].from - base_l) as u32;

        match offset_s.cmp(&offset_l) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                on_match(&edges_s[i], &edges_l[j], offset_s, i + 1)?;
                i += 1;
                j += 1;
            }
        }
    }
    Ok(())
}

fn out_chain(
    pre: &mut Vec<u8>,
    level: usize,
    edges: (&Edge, &Edge),
    trees: (&[Vec<Edge>], &[Vec<Edge>]),
    writer: &mut impl Write,
    count: &mut usize,
) -> Result<()> {
    if level == 0 {
        pre.push(b'\n');
        writer.write_all(pre)?;
        pre.pop();
        *count += 1;
        return Ok(());
    }

    let (es, el) = edges;
    let (tree_s, tree_l) = trees;

    let slice_s = &tree_s[level - 1][es.as_range()];
    let slice_l = &tree_l[level - 1][el.as_range()];

    find_intersections(
        (slice_s, slice_l),
        (es.to, el.to),
        |child_s, child_l, offset, _| {
            let original_len = pre.len();
            write!(pre, "->0x{:X}", offset)?;

            out_chain(pre, level - 1, (child_s, child_l), trees, writer, count)?;

            pre.truncate(original_len);
            Ok(())
        },
    )
}

fn process_areas_with_pb<F>(
    larger: &CoreData,
    smaller: &CoreData,
    mut process_area: F,
) -> Result<()>
where
    F: FnMut(usize, &Module, &ModuleArea, &ModuleArea, &ProgressBar) -> Result<()>,
{
    let depth = smaller.tree.len();

    let mut module_map: Vec<HashMap<&str, usize>> = vec![HashMap::new(); depth + 1];
    for (i, area) in larger.areas.iter().enumerate() {
        if area.level > depth {
            continue;
        }
        module_map[area.level].insert(larger.modules[area.index].name.as_str(), i);
    }

    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::with_template(
        "[{elapsed_precise:.yellow}] {prefix} 总数量: {msg}\n[{pos}/{len}] {bar:30.cyan/blue} ({percent}%) ETA: {eta:.cyan}",
    )?);

    for area_s in &smaller.areas {
        let level = area_s.level;
        let module_s = &smaller.modules[area_s.index];

        let Some(&index) = module_map[level].get(module_s.name.as_str()) else {
            continue;
        };
        let Some(area_l) = larger.areas.get(index) else {
            continue;
        };

        pb.set_length(area_s.edges.len() as u64);
        pb.set_position(0);
        pb.set_prefix(format!("层数:[{level}/{depth}] 模块:[{}]", module_s.name));

        process_area(level, module_s, area_s, area_l, &pb)?;
    }

    pb.finish_and_clear();
    Ok(())
}

pub fn compare_files(in_path1: &str, in_path2: &str, out_path: &str) -> Result<usize> {
    let read_timer = Instant::now();
    println!("开始读取...");

    let chains1 = format::read_chains_from_binary(in_path1)?;
    let chains2 = format::read_chains_from_binary(in_path2)?;

    let (larger, smaller) = if chains1.areas.len() >= chains2.areas.len() {
        (&chains1, &chains2)
    } else {
        (&chains2, &chains1)
    };

    println!("读取耗时：{:.4} 秒", read_timer.elapsed().as_secs_f64());

    let compare_timer = Instant::now();
    println!("开始对比");

    let mut count = 0;

    let mut writer = BufWriter::new(File::create(out_path)?);
    let mut buffer = Vec::with_capacity(1024);
    let mut last_flush = Instant::now();
    let flush_interval = Duration::from_secs(5);

    process_areas_with_pb(larger, smaller, |level, module_s, area_s, area_l, pb| {
        let offset_base_s = module_s.start;
        let offset_base_l = larger.modules[area_l.index].start;
        let name = &module_s.name;

        find_intersections(
            (&area_s.edges, &area_l.edges),
            (offset_base_s, offset_base_l),
            |es, el, offset, idx_s| -> Result<()> {
                buffer.clear();
                pb.set_position(idx_s as u64);
                pb.set_message(count.to_string());

                write!(&mut buffer, "{name}+0x{offset:X}")?;

                out_chain(
                    &mut buffer,
                    level,
                    (es, el),
                    (&smaller.tree, &larger.tree),
                    &mut writer,
                    &mut count,
                )?;

                if last_flush.elapsed() >= flush_interval {
                    writer.flush()?;
                    last_flush = Instant::now();
                }
                Ok(())
            },
        )?;
        Ok(())
    })?;

    writer.flush()?;

    if count == 0 {
        let _ = fs::remove_file(out_path);
    }

    println!("对比耗时：{:.4} 秒", compare_timer.elapsed().as_secs_f64());

    Ok(count)
}

pub fn format_file(in_path: &str, out_path: &str) -> Result<usize> {
    let chains = format::read_chains_from_binary(in_path)?;

    let file = File::create(out_path)?;
    let mut writer = BufWriter::new(file);

    fn out_chain(
        writer: &mut impl Write,
        chains: &CoreData,
        buffer: &mut String,
        level: usize,
        e: &Edge,
        count: &mut usize,
    ) -> Result<()> {
        if level == 0 {
            writer.write_all(buffer.as_bytes())?;
            writer.write_all(b"\n")?;
            *count += 1;
        } else {
            for i in e.as_range() {
                let original_len = buffer.len();

                let next_edge = &chains.tree[level - 1][i];

                let offset = next_edge.from.wrapping_sub(e.to);
                write!(buffer, "->0x{offset:X}")?;

                out_chain(writer, chains, buffer, level - 1, next_edge, count)?;

                buffer.truncate(original_len);
            }
        }

        Ok(())
    }

    let mut count = 0;
    let mut buffer = String::with_capacity(1024);
    for area in &chains.areas {
        let module = &chains.modules[area.index];

        for edge in &area.edges {
            buffer.clear();

            let offset = edge.from.wrapping_sub(module.start);

            write!(&mut buffer, "{name}+0x{offset:X}", name = module.name)?;

            out_chain(
                &mut writer,
                &chains,
                &mut buffer,
                area.level,
                edge,
                &mut count,
            )?;
        }
    }

    writer.flush()?;
    Ok(count)
}
