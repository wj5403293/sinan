use std::io::Result;
use std::mem::{MaybeUninit, size_of, size_of_val};
use std::slice::{from_raw_parts, from_raw_parts_mut};

cfg_select! {
    any(target_os = "linux", target_os = "android") => {
        mod linux;
        pub(crate) use linux::ProcessHandleInner;
    }
    target_os = "windows" => {
        mod windows;
        pub(crate) use windows::ProcessHandleInner;
    }
    target_os = "macos" => {
        mod macos;
        pub(crate) use macos::ProcessHandleInner;
    }
}

pub struct ProcessHandle {
    inner: ProcessHandleInner,
}

unsafe impl Send for ProcessHandle {}
unsafe impl Sync for ProcessHandle {}

impl ProcessHandle {
    pub fn new(pid: u32) -> Result<Self> {
        Ok(Self {
            inner: ProcessHandleInner::new(pid)?,
        })
    }

    #[inline(always)]
    pub fn read_val<T: Copy>(&self, addr: u64) -> Option<T> {
        let mut val = MaybeUninit::<T>::uninit();
        let slice = unsafe { from_raw_parts_mut(val.as_mut_ptr().cast(), size_of::<T>()) };
        if self.inner.read_bytes(addr, slice).is_ok() {
            Some(unsafe { val.assume_init() })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn write_val<T: Copy>(&self, addr: u64, val: &T) -> bool {
        let slice = unsafe { from_raw_parts((val as *const T).cast(), size_of::<T>()) };
        self.inner.write_bytes(addr, slice).is_ok()
    }

    #[inline(always)]
    pub fn read_array<T: Copy>(&self, addr: u64, buf: &mut [T]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let byte_len = size_of_val(buf);
        let slice = unsafe { from_raw_parts_mut(buf.as_mut_ptr().cast(), byte_len) };
        self.inner.read_bytes(addr, slice)
    }

    #[inline(always)]
    pub fn write_array<T: Copy>(&self, addr: u64, buf: &[T]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let byte_len = size_of_val(buf);
        let slice = unsafe { from_raw_parts(buf.as_ptr().cast(), byte_len) };
        self.inner.write_bytes(addr, slice)
    }

    pub fn read_pointer_chain(&self, addr: u64, offsets: &[usize]) -> Option<u64> {
        let mut current = addr;
        let limit = offsets.len().saturating_sub(1);

        for &offset in offsets.iter().take(limit) {
            current += offset as u64;
            current = self.read_val::<u64>(current)?;
        }

        if let Some(&last) = offsets.last() {
            current += last as u64;
        }

        Some(current)
    }
}
