use mach2::{
    kern_return,
    port::{mach_port_deallocate, mach_port_name_t},
    traps::{mach_task_self, task_for_pid},
    vm::{mach_vm_read_overwrite, mach_vm_write},
};
use std::io::{self, Result};

pub(crate) struct ProcessHandleInner {
    task: mach_port_name_t,
}

impl Drop for ProcessHandleInner {
    fn drop(&mut self) {
        unsafe {
            mach_port_deallocate(mach_task_self(), self.task);
        }
    }
}

impl ProcessHandleInner {
    pub(crate) fn new(pid: u32) -> Result<Self> {
        let mut task: mach_port_name_t = 0;
        let result = unsafe { task_for_pid(mach_task_self(), pid, &mut task) };
        if result != kern_return::KERN_SUCCESS {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { task })
    }

    #[inline(always)]
    pub(crate) fn read_bytes(&self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let mut read_len = 0;
        let result = unsafe {
            mach_vm_read_overwrite(
                self.task,
                addr as _,
                buf.len() as _,
                buf.as_mut_ptr() as _,
                &mut read_len,
            )
        };
        if result != kern_return::KERN_SUCCESS || read_len as usize != buf.len() {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub(crate) fn write_bytes(&self, addr: u64, buf: &[u8]) -> Result<()> {
        let result =
            unsafe { mach_vm_write(self.task, addr as _, buf.as_ptr() as _, buf.len() as _) };
        if result != kern_return::KERN_SUCCESS {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
