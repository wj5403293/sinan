pub mod maps;
pub mod mem;

use crate::errors::AppError;
use crate::proc::mem::ProcessHandle;

use anyhow::{Context, Result, bail};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

static SYSTEM: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new()));

#[derive(Clone, Debug)]
pub enum ProcessTarget {
    Name(String),
    Pid(u32),
}

#[derive(Default)]
struct ProcessState {
    target: Option<ProcessTarget>,
    pid: Option<u32>,
    handle: Option<Arc<ProcessHandle>>,
}

static STATE: LazyLock<RwLock<ProcessState>> =
    LazyLock::new(|| RwLock::new(ProcessState::default()));

pub fn set_target(input: &str) {
    let mut state = STATE.write().unwrap_or_else(|e| e.into_inner());

    if let Ok(pid) = input.parse::<u32>() {
        state.target = Some(ProcessTarget::Pid(pid));
    } else {
        state.target = Some(ProcessTarget::Name(input.to_string()));
    }

    state.pid = None;
    state.handle = None;
}

pub fn get_target_name() -> Option<String> {
    let state = STATE.read().unwrap_or_else(|e| e.into_inner());

    match &state.target {
        Some(ProcessTarget::Name(name)) => Some(name.clone()),
        Some(ProcessTarget::Pid(pid)) => Some(pid.to_string()),
        None => None,
    }
}

pub fn get_pid() -> Option<u32> {
    STATE.read().unwrap_or_else(|e| e.into_inner()).pid
}

pub fn get_handle() -> Option<Arc<ProcessHandle>> {
    STATE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .handle
        .clone()
}

pub fn ensure_attached() -> Result<(u32, Arc<ProcessHandle>)> {
    {
        let state = STATE.read().unwrap_or_else(|e| e.into_inner());
        if let (Some(pid), Some(handle)) = (state.pid, &state.handle)
            && is_pid_exist(pid)
        {
            return Ok((pid, handle.clone()));
        }
    }

    let mut state = STATE.write().unwrap_or_else(|e| e.into_inner());

    let target = state.target.clone().ok_or(AppError::TargetNotSet)?;
    let new_pid = match target {
        ProcessTarget::Pid(p) => {
            if is_pid_exist(p) {
                p
            } else {
                bail!(AppError::ProcessNotFound(p))
            }
        }
        ProcessTarget::Name(name) => {
            find_pid_by_name(&name).with_context(|| format!("无法找到名为 '{}' 的进程", name))?
        }
    };

    let handle = Arc::new(ProcessHandle::new(new_pid)?);

    state.pid = Some(new_pid);
    state.handle = Some(handle.clone());

    Ok((new_pid, handle))
}

fn find_pid_by_name(target_name: &str) -> Option<u32> {
    let mut sys = SYSTEM.lock().unwrap_or_else(|e| e.into_inner());

    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::Always)
            .with_memory(),
    );

    sys.processes()
        .values()
        .filter(|p| {
            p.cmd()
                .first()
                .and_then(|cmd| cmd.to_str())
                .map(|cmd| cmd.contains(target_name))
                .unwrap_or(false)
        })
        .max_by_key(|p| p.memory())
        .map(|p| p.pid().as_u32())
}

pub fn is_pid_exist(pid: u32) -> bool {
    let Ok(mut sys) = SYSTEM.lock() else {
        return false;
    };

    let s_pid = Pid::from(pid as usize);
    sys.refresh_processes(ProcessesToUpdate::Some(&[s_pid]), true);
    sys.process(s_pid).is_some()
}
