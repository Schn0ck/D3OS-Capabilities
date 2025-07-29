#![warn(missing_docs)]


use alloc::sync::Arc;
use bitflags::bitflags;
use spin::{Mutex, MutexGuard};

bitflags! {
    #[derive(Clone, Copy)]
    pub struct CapabilityFlags: u32 {
        const READ =     0b00000001;
        const WRITE =    0b00000010;
        const EXECUTE =  0b00000100;
        const SHARE =    0b00001000;
        const TRANSFER = 0b00010000;
    }
}

pub struct Capability<T> {
    obj: Option<Arc<Mutex<T>>>,
    flags: CapabilityFlags,
}

impl<T> Capability<T> {

    pub fn new(obj: T, flags: CapabilityFlags) -> Self {
        Self {
            obj: Some(Arc::new(Mutex::new(obj))),
            flags
        }
    }

    pub fn has_permissions(&self, flags: CapabilityFlags) -> bool {
        self.flags.contains(flags)
    }

    fn invoke_mut(&mut self) -> Option<spin::MutexGuard<'_, T>> {
        if self.has_permissions(CapabilityFlags::WRITE) {
            self.obj.as_ref().map(|arc| arc.lock())
        } else {
            None
        }
    }

    fn invoke(&self) -> Option<spin::MutexGuard<'_, T>> {
        if self.has_permissions(CapabilityFlags::READ) {
            self.obj.as_ref().map(|arc| arc.lock())
        } else {
            None
        }
    }


    pub fn share(&self, new_flags: CapabilityFlags) -> Option<Capability<T>> {
        if !self.has_permissions(CapabilityFlags::SHARE) {
            return None;
        }

        if !self.flags.contains(new_flags) {
            return None;
        }

        self.obj.as_ref().map(|arc| Capability {
            obj: Some(Arc::clone(arc)), //only clone the Arc, not the inner object
            flags: new_flags,
        })
    }



}
