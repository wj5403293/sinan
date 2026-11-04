pub mod map_vec;
pub mod numeric;

use std::path::Path;

pub fn for_each_block<F>(total_size: usize, chunk_size: usize, mut func: F)
where
    F: FnMut(usize, usize),
{
    let chunk_size = if chunk_size == 0 {
        total_size
    } else {
        chunk_size
    };

    let mut offset = 0;
    while offset < total_size {
        let current_chunk = chunk_size.min(total_size - offset);
        func(offset, current_chunk);
        offset += current_chunk;
    }
}

pub fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn get_incremented_filename(basename: &str, extension: &str) -> String {
    let mut new_name = format!("{basename}{extension}");
    let mut count = 0;

    while Path::new(&new_name).exists() {
        count += 1;
        new_name = format!("{basename}{count}{extension}");
    }

    new_name
}
