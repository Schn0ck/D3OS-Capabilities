#![warn(missing_docs)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::{Add, Deref};
use log::{info, warn};
use spin::Once;
use naming::shared_types::OpenOptions;
use syscall::NUM_SYSCALLS;
use crate::capabilities::capability;
use crate::capabilities::capability::Capability;
use crate::capabilities::capability_objects::naming_object::{create_naming_capability, NamingObject};
use crate::capabilities::capability_objects::syscall_object::Syscall;
use crate::naming::{api, lookup};
use crate::naming::api::shared_pipe;
use crate::naming::traits::{as_named_object, DirectoryObject, NamedObject};
use crate::syscall::sys_concurrent::*;
use crate::syscall::sys_naming::*;
use crate::syscall::sys_terminal::*;
use crate::syscall::sys_time::*;
use crate::syscall::sys_vmem::*;
use crate::syscall::sys_caps::*;
use crate::syscall::sys_net::*;

const BROADCAST_PIPE : Once<Capability<NamingObject>> = Once::new();

pub struct CSpace{
    syscall_capabilities: Vec<Capability<Syscall>>,
    naming_capabilities: Vec<Capability<NamingObject>>,
    //memory_capabilities: Vec<Capability<>>,
    //driver_capabilities: Vec<Capability<>>,
    //... other capability types
}

impl CSpace{ //TODO shared CSpace between all threads in a process? It is implemented but keep it??
    pub fn new() -> Self {
        let syscall_fns: [*const (); NUM_SYSCALLS] = [
            sys_terminal_read as *const (), //0
            sys_terminal_read_nb as *const (),
            sys_terminal_write as *const (),
            sys_map_memory as *const (),
            sys_map_frame_buffer as *const (),
            sys_process_execute_binary as *const (), //5
            sys_process_id as *const (),
            sys_process_exit as *const (),
            sys_thread_create as *const (),
            sys_thread_id as *const (),
            sys_thread_switch as *const (), //10
            sys_thread_sleep as *const (),
            sys_thread_join as *const (),
            sys_thread_exit as *const (),
            sys_get_system_time as *const (),
            sys_get_date as *const (), //15
            sys_set_date as *const (),
            sys_open as *const (),
            sys_read as *const (),
            sys_write as *const (),
            sys_seek as *const (), //20
            sys_close as *const (),
            sys_mkdir as *const (),
            sys_touch as *const (),
            sys_readdir as *const (),
            sys_cwd as *const (), //25
            sys_cd as *const (),
            sys_sock_open as *const (),
            sys_sock_bind as *const (),
            sys_sock_accept as *const (),
            sys_sock_connect as *const (), //30
            sys_sock_send as *const (),
            sys_sock_receive as *const (),
            sys_sock_close as *const (),
            sys_get_ip_adresses as *const (),
            sys_mkfifo as *const (), //35
            //caps
            sys_share_syscall_cap as *const (),
            sys_revoke_syscall_cap as *const (),
            sys_share_naming_cap as *const (),
            sys_naming_len as *const (),
        ]; //TODO individual configuration depending on calling app
        
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

        let mut naming_capabilities = Vec::new();
        //check if naming is initialized already
        if api::ROOT.is_completed() {
            if let Some(root) = api::ROOT.get(){
                let root_cap = api::root();
                let shared_pipe = shared_pipe(&root_cap);
                naming_capabilities.push(root_cap); //ROOT at index 0
                naming_capabilities.push(shared_pipe); //SHARED_PIPE at index 1
            }
            // naming_capabilities.push(api::root());
        }


        
        // if let Some(root) = api::ROOT.get(){ 
        //     let root_cap = create_naming_capability(NamedObject::from(root.root_dir()), OpenOptions::all(), None);//NamedObject::DirectoryObject(root.root_dir()), OpenOptions::all(), None);
        //     naming_capabilities.push(root_cap);
        // }
        
        Self {
            syscall_capabilities,
            naming_capabilities,
            //memory_capabilities: Vec::new(),
            //driver_capabilities: Vec::new(),
            //... initialize other capability types
        }
    }
    
    pub fn receive_syscall_capability(&mut self, capability: Option<Capability<Syscall>>, syscall_num: usize) -> isize{
        if let Some(capability) = capability {
            if let Some(cap) = self.syscall_capabilities.get_mut(syscall_num) {
                if let Some(combined) = capability.combine(cap){
                    self.syscall_capabilities[syscall_num] = combined
                } //else they dont point to the same syscall so keep current
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

    pub(crate) fn receive_root_naming_capability(&mut self, capability: Option<Capability<NamingObject>>) -> isize{
        if let Some(cap) = capability {
            self.naming_capabilities[0] = cap;
            return 0; //panic if len > isize::MAX (9_223_372_036_854_775_808) --> practically impossible
        }

        -1
    }

    pub fn receive_naming_capability(&mut self, capability: Option<Capability<NamingObject>>) -> isize{ //todo warum nochmal receive option
        if let Some(cap) = capability {
            info!("     CSpace: Naming capability is none: {}", cap.is_none());
            self.naming_capabilities.push(cap);
            info!("     CSpace: Received naming capability, new length {}", self.naming_capabilities.len());
            return self.naming_capabilities.len() as isize - 1; //panic if len > isize::MAX (9_223_372_036_854_775_808) --> practically impossible
        }

        warn!("     CSpace: Failed to receive naming capability");
        -1
    }

    pub fn get_naming_capability(&self, handle: usize) -> Option<&Capability<NamingObject>> {
        self.naming_capabilities.get(handle)
    }

    pub fn get_naming_capability_mut(&mut self, handle: usize) -> Option<&mut Capability<NamingObject>> {
        self.naming_capabilities.get_mut(handle)
    }

    pub fn get_naming_capabilities_len(&self) -> usize {
        self.naming_capabilities.len()
    }
    //
    // pub fn debug_print_caps(&self){
    //     info!("CSpace: Dumping syscall capabilities:");
    //     for (i, cap) in self.syscall_capabilities.iter().enumerate(){
    //         info!("    Syscall {}: {:?}", i, cap);
    //     }
    //     info!("CSpace: Dumping naming capabilities:");
    //     for (i, cap) in self.naming_capabilities.iter().enumerate(){
    //         info!("    Naming Cap {}: {:?}", i, cap);
    //     }
    // }
    
}