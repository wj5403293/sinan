use super::models::Pointer;

use crate::proc::{
    get_handle,
    maps::{VMArea, get_addr_range, is_addr_mapped},
};
use crate::utils::{self, map_vec::MapVec, numeric::val_to_bit_mask};

use anyhow::{Result, bail};
use rayon::prelude::*;
use std::cell::RefCell;

thread_local! {
    static TLS_BUFFER: RefCell<Vec<u32>> = RefCell::new(Vec::with_capacity((1 << 20) / 4 + 1));
}

fn filter_pointer_from_block(
    start: u64,
    offset: usize,
    read_size: usize,
    areas: &[VMArea],
) -> Result<Vec<Pointer>> {
    let Some(handle) = get_handle() else {
        bail!("Failed to get process handle");
    };

    let Some((min, max)) = get_addr_range(areas) else {
        bail!("Failed to get address range");
    };

    let bit_mask = val_to_bit_mask(max);

    let mut pvec = Vec::with_capacity(1024);
    TLS_BUFFER.with(|cell| {
        let read_len = read_size / 4 + 1;

        let mut buf = cell.borrow_mut();
        unsafe {
            buf.set_len(read_len);
        }

        let Ok(vaild_size) = handle.read_array(start + offset as u64, &mut buf[..read_len]) else {
            return;
        };

        let mut i = 0;
        while i + 1 < vaild_size / 4 {
            if buf[i] == 0 {
                i += 1;
                continue;
            }

            let (low, high) = (buf[i], buf[i + 1]);

            #[cfg(all(target_os = "android", target_arch = "aarch64"))]
            {
                let masked = high & 0xffff_ff00;
                if masked != 0x0000_0000 && masked != 0xb400_0000 {
                    i += 1;
                    continue;
                }
            }

            let value = (((high as u64) << 32) | (low as u64)) & bit_mask;
            if value > max || value < min {
                i += 1;
                continue;
            }

            if is_addr_mapped(areas, value) {
                pvec.push(Pointer {
                    address: start + (offset + i * 4) as u64,
                    value,
                });
                i += 2;
            } else {
                i += 1;
            }
        }
    });

    Ok(pvec)
}

pub fn get_pointers(areas: &[VMArea]) -> Result<MapVec<Pointer>> {
    let mut tasks = Vec::with_capacity(256);
    areas.iter().for_each(|area| {
        utils::for_each_block(area.size, 1 << 20, |offset, size| {
            tasks.push((area.start, offset, size));
        });
    });

    let results: Vec<Vec<Pointer>> = tasks
        .into_par_iter()
        .map(|(start, offset, size)| filter_pointer_from_block(start, offset, size, areas))
        .collect::<Result<Vec<_>, _>>()?;

    let mut pointers = MapVec::<Pointer>::new()?;
    for pvec in results.into_iter() {
        if pvec.is_empty() {
            continue;
        }
        pointers.write_slice(&pvec)?;
    }
    pointers.map()?;

    Ok(pointers)
}
