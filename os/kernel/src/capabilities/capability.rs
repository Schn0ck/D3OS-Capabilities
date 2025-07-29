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

    fn invoke(&self) -> Option<spin::MutexGuard<'_, T>> {
        if self.has_permissions(CapabilityFlags::READ) {
            self.obj.as_ref().map(|arc| arc.lock())
        } else {
            None
        }
    }

    fn invoke_mut(&mut self) -> Option<spin::MutexGuard<'_, T>> {
        if self.has_permissions(CapabilityFlags::READ | CapabilityFlags::WRITE) {
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

    pub fn transfer(&mut self, new_flags: CapabilityFlags) -> Option<Capability<T>> {
        if !self.has_permissions(CapabilityFlags::TRANSFER) {
            return None;
        }

        if !self.flags.contains(new_flags) {
            return None;
        }

        let new_cap = self.obj.as_ref().map(|arc| Capability {
            obj: Some(Arc::clone(arc)), //only clone the Arc, not the inner object 
            flags: new_flags,
        });

        if new_cap.is_some() {
            self.revoke();
        }

        new_cap
    }
    
    pub fn revoke(&mut self) {
        self.obj = None;
        self.flags = CapabilityFlags::empty();
    }
}

// Hilfreiche Methoden für die Erstellung von Capabilities mit verschiedenen Berechtigungen
impl<T> Capability<T> {
    pub fn readonly(obj: T) -> Self {
        Self::new(obj, CapabilityFlags::READ)
    }

    pub fn readwrite(obj: T) -> Self {
        Self::new(obj, CapabilityFlags::READ | CapabilityFlags::WRITE)
    }

    pub fn shareable(obj: T) -> Self {
        Self::new(obj, CapabilityFlags::READ | CapabilityFlags::SHARE)
    }

    pub fn transferable(obj: T) -> Self {
        Self::new(obj, CapabilityFlags::READ | CapabilityFlags::TRANSFER)
    }

    pub fn full_access(obj: T) -> Self {
        Self::new(obj, CapabilityFlags::all())
    }

    pub fn null() -> Self {
        Self { obj: None, flags: CapabilityFlags::empty()}
    }
}

