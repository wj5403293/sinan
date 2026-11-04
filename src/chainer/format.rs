use super::models::*;

use anyhow::{Result, bail};
use bincode::config;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

pub fn write_chains_to_binary(chains: &CoreData, filepath: &str) -> Result<usize> {
    let file = File::create(filepath)?;

    let config = config::standard();
    let mut bytes = 0;

    let mut writer = BufWriter::new(file);
    bytes += bincode::encode_into_std_write(Header::default(), &mut writer, config)?;
    bytes += bincode::encode_into_std_write(chains, &mut writer, config)?;

    writer.flush()?;

    Ok(bytes)
}

pub fn read_chains_from_binary(filepath: &str) -> Result<CoreData> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);

    let config = config::standard();

    let header: Header = bincode::decode_from_std_read(&mut reader, config)?;

    if header.magic != MAGIC {
        bail!("Invalid MAGIC header");
    }
    if header.arch_bit != ARCH_BIT {
        bail!("Mismatched ARCH_BIT");
    }
    if header.version != VERSION {
        bail!("Mismatched VERSION");
    }

    let chains: CoreData = bincode::decode_from_std_read(&mut reader, config)?;

    Ok(chains)
}

pub fn calculate_pointer_chain_count(areas: &[ModuleArea], tree: &[Vec<Edge>]) -> usize {
    let mut counts: Vec<Vec<usize>> = Vec::with_capacity(tree.len() + 1);

    counts.push(vec![0, 1]);

    for level in 1..=tree.len() {
        let mut count = 0;
        let mut level_counts = vec![0];

        for edge in &tree[level - 1] {
            count += counts[level - 1][(edge.start + edge.size) as usize]
                - counts[level - 1][edge.start as usize];
            level_counts.push(count);
        }

        counts.push(level_counts);
    }

    let mut total_count = 0;
    for area in areas {
        for edge in &area.edges {
            total_count += counts[area.level][(edge.start + edge.size) as usize]
                - counts[area.level][edge.start as usize];
        }
    }

    total_count
}
