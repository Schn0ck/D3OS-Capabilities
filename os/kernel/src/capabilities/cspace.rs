#![warn(missing_docs)]


use alloc::vec::Vec;
use log::info;
use syscall::NUM_SYSCALLS;
use crate::capabilities::capability::Capability;
use crate::capabilities::capability_objects::{NamingObject, Syscall};
use crate::syscall::sys_concurrent::*;
use crate::syscall::sys_naming::*;
use crate::syscall::sys_terminal::*;
use crate::syscall::sys_time::*;
use crate::syscall::sys_vmem::*;
use crate::syscall::sys_caps::*;
use crate::syscall::sys_net::*;

pub struct CSpace {
    syscall_capabilities: Vec<Capability<Syscall>>,
    naming_capabilities: Vec<Capability<NamingObject>>,
    //memory_capabilities: Vec<Capability<>>,
    //driver_capabilities: Vec<Capability<>>,
    //... other capability types
}

impl CSpace {
    pub fn new() -> Self {
        let syscall_fns: [*const (); NUM_SYSCALLS] = [
            sys_terminal_read as *const (),
            sys_terminal_read_nb as *const (),
            sys_terminal_write as *const (),
            sys_map_memory as *const (),
            sys_map_frame_buffer as *const (),
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
            sys_sock_open as *const (),
            sys_sock_bind as *const (),
            sys_sock_accept as *const (),
            sys_sock_connect as *const (),
            sys_sock_send as *const (),
            sys_sock_receive as *const (),
            sys_sock_close as *const (),
            sys_get_ip_adresses as *const (),
            sys_mkfifo as *const (),
            //caps
            sys_share_syscall_cap as *const (),
            sys_revoke_syscall_cap as *const (),
        ];
        
        let mut num = 0;
        let mut syscall_capabilities: Vec<_> = syscall_fns
            .iter()
            .map(|&f| {
                let cap = Capability::syscall(Syscall::new(num, f));
                num += 1;
                cap
            })
            .collect();

             // Example of revoking a specific syscall capability
         if let Some(mut cap) = syscall_capabilities.get_mut(13) {
            //cap.revoke();
        }
        
        Self {
            syscall_capabilities ,
            naming_capabilities: Vec::new(),
            //memory_capabilities: Vec::new(),
            //driver_capabilities: Vec::new(),
            //... initialize other capability types
        }
    }
    
    //TODO implement methods to add, remove, and manage capabilities
    pub fn receive_syscall_capability(&mut self, capability: Option<Capability<Syscall>>, syscall_num: usize) -> isize{
        if let Some(capability) = capability {
            if let Some(cap) = self.syscall_capabilities.get_mut(syscall_num) {
                cap.add_permissions(capability.get_permissions())
            } else {
                self.syscall_capabilities[syscall_num] = capability;
                
            }
            return syscall_num.try_into().unwrap(); //panics if syscall num > isize::MAX (9_223_372_036_854_775_808) --> practically impossible
        }
        -1
    }
    
    pub fn revoke_syscall_capability(&mut self, syscall_num: usize){
        if let Some(cap) = self.syscall_capabilities.get_mut(syscall_num) {
            cap.revoke();
        }
    }
    pub fn get_syscall_capability(&self, syscall_num: usize) -> Option<&Capability<Syscall>> {
        self.syscall_capabilities.get(syscall_num)
    }
    pub fn get_syscall_capability_mut(&mut self, syscall_num: usize) -> Option<&mut Capability<Syscall>> {
        self.syscall_capabilities.get_mut(syscall_num)
    }
    pub fn remove_syscall_capability(&mut self, syscall_num: usize) -> Capability<Syscall> {
        self.syscall_capabilities.remove(syscall_num)
    }
    
    pub fn receive_naming_capability(&mut self, capability: Option<Capability<NamingObject>>) -> isize{ //TODO more than append only?
        if let Some(capability) = capability {
            self.naming_capabilities.push(capability);
            return self.naming_capabilities.len() as isize - 1; //panic if len > isize::MAX (9_223_372_036_854_775_808) --> practically impossible
        }
        
        -1
    }

    pub fn get_naming_capability(&self, handle: usize) -> Option<&Capability<NamingObject>> {
        self.naming_capabilities.get(handle)
    }

    pub fn get_naming_capability_mut(&mut self, handle: usize) -> Option<&mut Capability<NamingObject>> {
        self.naming_capabilities.get_mut(handle)
    }

}