#![warn(missing_docs)]


use alloc::sync::Arc;
use bitflags::bitflags;
use log::warn;
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
    pub(crate) fn is_none(&self) -> bool {
        self.obj.is_none()
    }
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

    pub fn get_permissions(&self) -> CapabilityFlags {
        self.flags
    }
    
    pub fn invoke(&self) -> Option<MutexGuard<'_, T>> {
        if !self.flags.contains(CapabilityFlags::READ) {
            warn!("Tried to invoke a capability without READ permission");
            return None;
        }

        self.obj.as_ref()?.try_lock()

        // if let Some(content) = self.obj.as_ref() {
        //     return content.try_lock(); //old: .map(|arc| arc.lock())
        // }
        // None
    }

    // pub fn invoke_mut(&mut self) -> Option<MutexGuard<'_, T>> {
    //     if !self.flags.contains(CapabilityFlags::READ| CapabilityFlags::WRITE) {
    //         return None;
    //     }
    //     
    //     self.obj.as_mut()?.lock()
    // } Not needed due to Arc


    pub fn share(&self, new_flags: CapabilityFlags) -> Option<Capability<T>> {
        if !self.has_permissions(CapabilityFlags::SHARE) {
            return None;
        }

        self.obj.as_ref().map(|arc| Capability {
            obj: Some(Arc::clone(arc)), 
            flags: new_flags & self.flags, //Share with less or equal permissions
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
            obj: Some(Arc::clone(arc)), 
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
    
    pub fn combine(&self, other: &Capability<T>) -> Option<Capability<T>> {
        // Only allow combining if both capabilities refer to the same object
        if let Some(obj) = &self.obj {
            if let Some(other_obj) = &other.obj {
                if Arc::<Mutex<T>>::as_ptr(obj) == Arc::<Mutex<T>>::as_ptr(other_obj) {
                    return Some(Capability {
                        obj: self.obj.clone(),
                        flags: self.flags.clone() | other.flags.clone(),
                    });
                }
            } 
        }
        None
    }

    // pub fn add_permissions(&mut self, flags: CapabilityFlags) {
    //     self.flags.insert(flags);
    // }
    // FORBIDDEN. Get rights by share or transfer!!!
    // Could lead to rights escalation.

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
    
    pub fn syscall(obj: T) -> Self {
        Self::new(obj, CapabilityFlags::READ | CapabilityFlags::EXECUTE | CapabilityFlags::SHARE)
    }
}