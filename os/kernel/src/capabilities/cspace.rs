#![warn(missing_docs)]


use alloc::vec::Vec;
use log::info;
use syscall::NUM_SYSCALLS;
use crate::capabilities::capability::{Capability};
use crate::syscall::sys_concurrent::{sys_process_execute_binary, sys_process_exit, sys_process_id, sys_thread_create, sys_thread_exit, sys_thread_id, sys_thread_join, sys_thread_sleep, sys_thread_switch};
use crate::syscall::sys_naming::{sys_cd, sys_close, sys_cwd, sys_mkdir, sys_open, sys_read, sys_readdir, sys_seek, sys_touch, sys_write};
use crate::syscall::sys_terminal::{sys_terminal_read, sys_terminal_write};
use crate::syscall::sys_time::{sys_get_date, sys_get_system_time, sys_set_date};
use crate::syscall::sys_vmem::sys_map_memory;

pub struct CSpace {
    syscall_capabilities: Vec<Capability<Syscall>>,
    //memory_capabilities: Vec<Option<Capability<>>>,
    //ipc_capabilities: Vec<Option<Capability<>>>,
    //... other capability types
}

impl CSpace {
    pub fn new() -> Self {
        let syscall_fns: [*const (); NUM_SYSCALLS] = [
            sys_terminal_read as *const (),
            sys_terminal_write as *const (),
            sys_map_memory as *const (),
            sys_process_execute_binary as *const (),
            sys_process_id as *const (),
            sys_process_exit as *const (),
            sys_thread_create as *const (),
            sys_thread_id as *const (),
            sys_thread_switch as *const (),
            sys_thread_sleep as *const (),
            sys_thread_join as *const (),
            sys_thread_exit as *const (),
            sys_get_system_time as *const (),
            sys_get_date as *const (),
            sys_set_date as *const (),
            sys_open as *const (),
            sys_read as *const (),
            sys_write as *const (),
            sys_seek as *const (),
            sys_close as *const (),
            sys_mkdir as *const (),
            sys_touch as *const (),
            sys_readdir as *const (),
            sys_cwd as *const (),
            sys_cd as *const (),
        ];

        let mut syscall_capabilities: Vec<_> = syscall_fns
            .iter()
            .map(|&f| Capability::readonly(Syscall::new(f)))
            .collect();

        /*        // Example of revoking a specific syscall capability
        if let Some(mut cap) = syscall_capabilities.get_mut(13) {
            cap.revoke();
        }
        */
        
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

pub struct Syscall {
    function: *const (),
}

impl Syscall {
    pub fn new(function: *const ()) -> Self {
        Self { function }
    }

    pub fn function_pointer(&self) -> *const () {
        self.function
    }
}


unsafe impl Send for Syscall {}
unsafe impl Sync for Syscall {}