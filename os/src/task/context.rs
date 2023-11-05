//! Implementation of [`TaskContext`]
use alloc::string::String;
use crate::loader::get_app_data_by_name;
use crate::task::{current_task, exit_current_and_run_next};
use crate::trap::trap_return;

#[repr(C)]
/// task context structure containing some registers
pub struct TaskContext {
    /// Ret position after task switching
    ra: usize,
    /// Stack pointer
    sp: usize,
    /// s0-11 register, callee saved
    s: [usize; 12],
}

impl TaskContext {
    /// Create a new empty task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    /// Create a new task context with a trap return addr and a kernel stack pointer
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }

    /// 11
    pub fn goto_trap_exec(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_exec as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}

/// 11
fn trap_exec() {
    let path : String;

    {
        let task = current_task().unwrap();
        let path_temp = &task.inner_exclusive_access().cmd_path;
        path = String::from(path_temp.as_str());
    }

    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        trap_return()
    }

    // kill
    println!("trap_exec exec err, ptah : {}", path);
    exit_current_and_run_next(1);
}