use crate::constants::{APP_ROOT, DIR_TMP};

use memmap2::{MmapMut, MmapOptions};
use std::fs;
use std::io::{self, BufWriter, Seek, Write};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tempfile::{Builder, NamedTempFile};

pub struct MapVec<T> {
    file: BufWriter<NamedTempFile>,
    mmap: Option<MmapMut>,
    mapped_size: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy> MapVec<T> {
    pub fn new() -> io::Result<Self> {
        let tmp_dir = Path::new(APP_ROOT).join(DIR_TMP);

        if !tmp_dir.exists() {
            fs::create_dir_all(&tmp_dir)?;
        }

        let temp_file = Builder::new().prefix("tmp_").tempfile_in(&tmp_dir)?;
        let buf_writer = BufWriter::with_capacity(1 << 20, temp_file);

        Ok(Self {
            file: buf_writer,
            mmap: None,
            mapped_size: 0,
            _marker: PhantomData,
        })
    }

    pub fn news(count: usize) -> io::Result<Vec<Self>> {
        let mut vecs = Vec::with_capacity(count);
        for _ in 0..count {
            vecs.push(Self::new()?);
        }
        Ok(vecs)
    }

    pub fn write_single(&mut self, value: &T) -> io::Result<usize> {
        self.write_slice(std::slice::from_ref(value))
    }

    pub fn write_slice(&mut self, data: &[T]) -> io::Result<usize> {
        if self.mmap.is_some() {
            return Err(io::Error::other("Cannot write after mapping"));
        }

        self.mapped_size += mem::size_of_val(data);
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, mem::size_of_val(data))
        };
        self.file.write_all(bytes)?;

        Ok(data.len())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }

    pub fn rewind(&mut self) -> io::Result<()> {
        self.file.rewind()
    }

    pub fn size(&self) -> usize {
        self.mapped_size / mem::size_of::<T>()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn map(&mut self) -> io::Result<()> {
        self.flush()?;

        let file = self.file.get_ref().as_file();
        if self.mapped_size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot map empty file",
            ));
        }
        let mmap = unsafe { MmapOptions::new().map_mut(file)? };

        self.mmap = Some(mmap);
        Ok(())
    }

    pub fn unmap(&mut self) {
        self.mmap = None;
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.unmap();

        let tmp_dir = Path::new(APP_ROOT).join(DIR_TMP);
        let temp_file = Builder::new().prefix("tmp_").tempfile_in(tmp_dir)?;
        self.file = BufWriter::with_capacity(1 << 20, temp_file);

        Ok(())
    }

    pub fn handle(&self) -> &std::fs::File {
        self.file.get_ref().as_file()
    }
}

impl<T> Deref for MapVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        if let Some(mmap) = &self.mmap {
            unsafe {
                std::slice::from_raw_parts(
                    mmap.as_ptr() as *const T,
                    mmap.len() / mem::size_of::<T>(),
                )
            }
        } else {
            &[]
        }
    }
}

impl<T> DerefMut for MapVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(mmap) = &mut self.mmap {
            unsafe {
                std::slice::from_raw_parts_mut(
                    mmap.as_mut_ptr() as *mut T,
                    mmap.len() / mem::size_of::<T>(),
                )
            }
        } else {
            &mut []
        }
    }
}
