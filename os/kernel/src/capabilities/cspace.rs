#![warn(missing_docs)]


use alloc::vec::Vec;
use syscall::NUM_SYSCALLS;
use crate::capabilities::capability::{Capability};
use crate::syscall::sys_concurrent::{sys_process_execute_binary, sys_process_exit, sys_process_id, sys_thread_create, sys_thread_exit, sys_thread_id, sys_thread_join, sys_thread_sleep, sys_thread_switch};
use crate::syscall::sys_naming::{sys_cd, sys_close, sys_cwd, sys_mkdir, sys_open, sys_read, sys_readdir, sys_seek, sys_touch, sys_write};
use crate::syscall::sys_terminal::{sys_terminal_read, sys_terminal_write};
use crate::syscall::sys_time::{sys_get_date, sys_get_system_time, sys_set_date};
use crate::syscall::sys_vmem::sys_map_memory;
use crate::syscall::syscall_dispatcher::{Syscall};

pub struct CSpace {
    syscall_capabilities: Vec<Capability<Syscall>>,
    //memory_capabilities: Vec<Option<Capability<>>>,
    //ipc_capabilities: Vec<Option<Capability<>>>,
    //... other capability types
}

impl CSpace {
    pub fn new() -> Self {
        let syscall_fns: [*const (); NUM_SYSCALLS] = [
            sys_terminal_read as *const _,
            sys_terminal_write as *const _,
            sys_map_memory as *const _,
            sys_process_execute_binary as *const _,
            sys_process_id as *const _,
            sys_process_exit as *const _,
            sys_thread_create as *const _,
            sys_thread_id as *const _,
            sys_thread_switch as *const _,
            sys_thread_sleep as *const _,
            sys_thread_join as *const _,
            sys_thread_exit as *const _,
            sys_get_system_time as *const _,
            sys_get_date as *const _,
            sys_set_date as *const _,
            sys_open as *const _,
            sys_read as *const _,
            sys_write as *const _,
            sys_seek as *const _,
            sys_close as *const _,
            sys_mkdir as *const _,
            sys_touch as *const _,
            sys_readdir as *const _,
            sys_cwd as *const _,
            sys_cd as *const _,
        ];

        let syscall_capabilities: Vec<_> = syscall_fns
            .iter()
            .map(|&f| Capability::readonly(Syscall::new(f)))
            .collect();
        
        
        Self {
            syscall_capabilities ,
            //memory_capabilities: Vec::new(),
            //ipc_capabilities: Vec::new(),
            //... initialize other capability types
        }
    }
    
    //TODO implement methods to add, remove, and manage capabilities
    
    pub fn add_syscall_capability(&mut self, capability: Capability<Syscall>) {
        self.syscall_capabilities.push(capability);
    }
    pub fn get_syscall_capability(&self, index: usize) -> Option<&Capability<Syscall>> {
        self.syscall_capabilities.get(index)
    }
    pub fn get_syscall_capability_mut(&mut self, index: usize) -> Option<&mut Capability<Syscall>> {
        self.syscall_capabilities.get_mut(index)
    }
    pub fn remove_syscall_capability(&mut self, index: usize) -> Capability<Syscall> {
        self.syscall_capabilities.remove(index)
    }
}