//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, get_curr_task,
    }, mm::{VirtAddr, MapPermission},
};
use crate::mm::{translated_user_addr};
use crate::task::{current_user_token, get_running_time, get_sys_call_count, get_task_status};
use crate::timer::{get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let pa = translated_user_addr(current_user_token(), ts as *const u8);
    if let Some(pa) = pa {
        let ts: *mut TimeVal = pa.get_mut();
        unsafe {
            let us = get_time_us();
            (*ts).sec = us / 1_000_000;
            (*ts).usec = us % 1_000_000;
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let pa = translated_user_addr(current_user_token(), ti as *const u8);
    if let Some(pa) = pa {
        let ti: *mut TaskInfo = pa.get_mut();
        unsafe {
            (*ti).status = get_task_status();
            (*ti).syscall_times = get_sys_call_count();

            let us = get_time_us();
            let sec = us / 1_000_000;
            let usec = us % 1_000_000;
            let t = (sec & 0xffff) * 1000 + usec / 1000;
            (*ti).time = t - get_running_time();
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1
    }

    let mut len = len;
    if len % PAGE_SIZE > 0 {
        len += PAGE_SIZE;
        len -= len % PAGE_SIZE;
    }

    let curr_task = get_curr_task();
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    let mut permission = MapPermission::U;
    if port & 1 > 0 {
        permission |= MapPermission::R;
    }
    if port & 2 > 0 {
        permission |= MapPermission::W;
    }
    if port & 4 > 0 {
        permission |= MapPermission::X;
    }

    // permission err
    if (port - (port & 7)) > 0 || permission == MapPermission::U {
        return -1;
    }

    match curr_task.memory_set.mmap_allocate_area(start_va, end_va, permission) {
        Ok(_) => 0,
        Err(_) => -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1
    }

    let curr_task = get_curr_task();
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    match curr_task.memory_set.unmap_free_area(start_va, end_va) {
        Ok(_) => 0,
        Err(_) => -1
    }
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
