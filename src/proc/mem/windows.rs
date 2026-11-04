use std::io::{self, Result};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
    System::Threading::{OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE},
};

pub(crate) struct ProcessHandleInner {
    handle: HANDLE,
}

impl Drop for ProcessHandleInner {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl ProcessHandleInner {
    pub(crate) fn new(pid: u32) -> Result<Self> {
        let handle = unsafe {
            OpenProcess(
                PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
                0, // false
                pid as _,
            )
        };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle })
    }

    #[inline(always)]
    pub(crate) fn read_bytes(&self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let mut bytes_read = 0;
        let res = unsafe {
            ReadProcessMemory(
                self.handle,
                addr as _,
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut bytes_read,
            )
        };
        if res == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub(crate) fn write_bytes(&self, addr: u64, buf: &[u8]) -> Result<()> {
        let mut bytes_written = 0;
        let res = unsafe {
            WriteProcessMemory(
                self.handle,
                addr as _,
                buf.as_ptr().cast(),
                buf.len(),
                &mut bytes_written,
            )
        };
        if res == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
