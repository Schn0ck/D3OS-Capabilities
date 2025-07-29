#![warn(missing_docs)]


use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use crate::capabilities::capability::{Capability, CapabilityFlags};

pub struct Syscall{}

pub struct CSpace {
    syscall_capabilities: Vec<Option<Capability<Syscall>>>,
    //memory_capabilities: Vec<Option<Capability<>>>,
    //... other capability types
}

impl CSpace {
    pub fn new() -> Self {
        Self {
            syscall_capabilities: Vec::new(),
            //memory_capabilities: Vec::new(),
            //... initialize other capability types
        }
    }
    
    //TODO implement methods to add, remove, and manage capabilities
    
    pub fn add_syscall_capability(&mut self, capability: Capability<Syscall>) {
        self.syscall_capabilities.push(Some(capability));
    }
    pub fn get_syscall_capability(&self, index: usize) -> Option<&Capability<Syscall>> {
        self.syscall_capabilities.get(index).and_then(|cap| cap.as_ref())
    }
    pub fn get_syscall_capability_mut(&mut self, index: usize) -> Option<&mut Capability<Syscall>> {
        self.syscall_capabilities.get_mut(index).and_then(|cap| cap.as_mut())
    }
    pub fn remove_syscall_capability(&mut self, index: usize) -> Option<Capability<Syscall>> {
        if index < self.syscall_capabilities.len() {
            self.syscall_capabilities.remove(index)
        } else {
            None
        }
    }
}