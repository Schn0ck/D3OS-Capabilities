use alloc::string::String;
use alloc::sync::Arc;
use core::sync::atomic::AtomicUsize;
use naming::shared_types::OpenOptions;
use crate::capabilities::capability::{Capability, CapabilityFlags};
use crate::naming::traits::NamedObject;

pub struct NamingObject {
    pub(crate)named_object: Arc<NamedObject>,
    pub(crate)access_rights: OpenOptions,
    pub(crate)position: AtomicUsize,
    pub(crate)path: String,
    parent_handle: Option<usize>,  // Handle to parent directory's capability TODO needed?
}

#[derive(Copy, Clone)]
pub enum ObjectType {
    File,
    Directory,
    Pipe,
}

impl NamingObject {
    fn new(object: NamedObject, rights: OpenOptions, parent: Option<usize>, path: String) -> Self {
        Self {
            named_object: Arc::from(object),
            access_rights: rights,
            position: AtomicUsize::new(0),
            parent_handle: parent,
            path,
        }
    }
}

pub fn create_naming_capability(object: NamedObject, rights: OpenOptions, parent: Option<usize>, path: String) -> Capability<NamingObject> {
    let mut flags = CapabilityFlags::empty();
    if rights.contains(OpenOptions::READONLY) || rights.contains(OpenOptions::READWRITE) {
        flags |= CapabilityFlags::READ;
    }
    if rights.contains(OpenOptions::WRITEONLY) || rights.contains(OpenOptions::READWRITE) {
        flags |= CapabilityFlags::WRITE;
    }
    if rights.contains(OpenOptions::SHARE) {
        flags |= CapabilityFlags::SHARE;
    }

    Capability::new(NamingObject::new(object, rights, parent, path), flags) //Todo make it customizable
}
