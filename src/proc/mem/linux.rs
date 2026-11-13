use libc::{iovec, pid_t, process_vm_readv, process_vm_writev};
use std::fs::File;
use std::io::Result;
use std::os::unix::fs::FileExt;

pub(crate) struct ProcessHandleInner {
    pid: pid_t,
    mem_file: File,
}

impl ProcessHandleInner {
    pub(crate) fn new(pid: u32) -> Result<Self> {
        let mem_file = File::options()
            .read(true)
            .write(true)
            .open(format!("/proc/{}/mem", pid))?;

        Ok(Self {
            pid: pid as pid_t,
            mem_file,
        })
    }

    #[inline(always)]
    fn translate_addr(&self, addr: u64) -> u64 {
        // Fallback: /proc/pid/mem
        if cfg!(all(target_os = "android", target_arch = "aarch64")) && (addr >> 40 == 0xb4_0000) {
            addr & 0xFF_FFFF
        } else {
            addr
        }
    }

    #[inline(always)]
    pub(crate) fn read_bytes(&self, addr: u64, buf: &mut [u8]) -> Result<usize> {
        let local_iov = iovec {
            iov_base: buf.as_mut_ptr().cast(),
            iov_len: buf.len(),
        };
        let remote_iov = iovec {
            iov_base: addr as _,
            iov_len: buf.len(),
        };

        let res = unsafe { process_vm_readv(self.pid, &local_iov, 1, &remote_iov, 1, 0) };

        if res >= 0 {
            return Ok(res as usize);
        }

        let final_addr = self.translate_addr(addr);
        self.mem_file.read_at(buf, final_addr)
    }

    #[inline(always)]
    pub(crate) fn write_bytes(&self, addr: u64, buf: &[u8]) -> Result<usize> {
        let local_iov = iovec {
            iov_base: buf.as_ptr() as _,
            iov_len: buf.len(),
        };
        let remote_iov = iovec {
            iov_base: addr as _,
            iov_len: buf.len(),
        };

        let res = unsafe { process_vm_writev(self.pid, &local_iov, 1, &remote_iov, 1, 0) };

        if res >= 0 {
            return Ok(res as usize);
        }

        let final_addr = self.translate_addr(addr);
        self.mem_file.write_at(buf, final_addr)
    }
}
