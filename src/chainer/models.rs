use bincode::{Decode, Encode};
use std::ops::Range;

pub const MAGIC: [u8; 4] = *b"SNBD";
pub const ARCH_BIT: u16 = 64;
pub const VERSION: u16 = 2;

#[derive(Debug, Clone, Copy, Default, Encode, Decode, PartialEq, Eq)]
pub struct Pointer {
    pub address: u64,
    pub value: u64,
}

#[derive(Debug, Clone, Default, Encode, Decode, PartialEq, Eq)]
pub struct Module {
    pub start: u64,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Default, Encode, Decode, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub start: u32,
    pub size: u32,
}

impl Edge {
    #[inline]
    pub fn as_range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start + self.size as usize;
        start..end
    }
}

#[derive(Debug, Clone, Default, Encode, Decode, PartialEq, Eq)]
pub struct ModuleArea {
    pub level: usize,
    pub index: usize,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Default, Encode, Decode, PartialEq, Eq)]
pub struct CoreData {
    pub modules: Vec<Module>,
    pub areas: Vec<ModuleArea>,
    pub tree: Vec<Vec<Edge>>,
}

#[derive(Debug, Encode, Decode)]
pub struct Header {
    pub magic: [u8; 4],
    pub arch_bit: u16,
    pub version: u16,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: MAGIC,
            arch_bit: ARCH_BIT,
            version: VERSION,
        }
    }
}
