use anyhow::{Context, Result};
use proc_maps::{Pid, get_process_maps};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VMArea {
    pub start: u64,
    pub end: u64,
    pub size: usize,
    pub name: String,
    pub is_read: bool,
    pub is_write: bool,
    pub is_exec: bool,
}

pub fn is_addr_mapped(areas: &[VMArea], addr: u64) -> bool {
    areas
        .binary_search_by(|entry| {
            if addr < entry.start {
                Ordering::Greater
            } else if addr >= entry.end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .is_ok()
}

pub fn get_total_size(areas: &[VMArea]) -> usize {
    areas.iter().map(|area| area.size).sum()
}

pub fn get_min_addr(areas: &[VMArea]) -> Option<u64> {
    areas.first().map(|area| area.start)
}

pub fn get_max_addr(areas: &[VMArea]) -> Option<u64> {
    areas.last().map(|area| area.end)
}

pub fn get_addr_range(areas: &[VMArea]) -> Option<(u64, u64)> {
    Some((areas.first()?.start, areas.last()?.end))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Maps {
    pub areas: Vec<VMArea>,
}

impl Maps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            areas: Vec::with_capacity(capacity),
        }
    }

    // NOTE: proc-maps 存在 Android 兼容性问题：
    // 对于 maps 中的零长度映射记录（start == end，例如 [page size compat]），
    // 库内部会因 start >= end 判定为非法参数并返回 EINVAL，
    // 导致整个 maps 解析失败。因此需要修改 /.cargo/registry/src/index.crates.io-*/proc-maps-0.5.0/src/linux_maps.rs 判断或自行解析 /proc/{pid}/maps，
    // 并跳过此类无效长度的映射记录。
    // 后续考虑修复此问题
    pub fn parse(pid: u32) -> Result<Self> {
        let maps = get_process_maps(pid as Pid)
            .with_context(|| format!("获取进程 maps 失败 (pid={pid})"))?;

        let mut areas = Self::new();
        for map in maps {
            areas.push(VMArea {
                start: map.start() as u64,
                end: (map.start() + map.size()) as u64,
                size: map.size(),
                name: map
                    .filename()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                is_read: map.is_read(),
                is_write: map.is_write(),
                is_exec: map.is_exec(),
            })
        }

        Ok(areas)
    }

    pub fn total_size(&self) -> usize {
        get_total_size(self)
    }

    pub fn min_addr(&self) -> Option<u64> {
        get_min_addr(self)
    }

    pub fn max_addr(&self) -> Option<u64> {
        get_max_addr(self)
    }
}

impl Deref for Maps {
    type Target = Vec<VMArea>;

    fn deref(&self) -> &Self::Target {
        &self.areas
    }
}

impl DerefMut for Maps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.areas
    }
}
