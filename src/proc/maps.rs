use anyhow::Result;
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

    pub fn parse(pid: u32) -> Result<Self> {
        let maps = get_process_maps(pid as Pid)?;

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
