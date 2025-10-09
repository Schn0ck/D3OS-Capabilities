use naming::shared_types::OpenOptions;
use crate::capabilities::capability::{Capability, CapabilityFlags};

pub struct Syscall {
    number: usize,
    function: *const (),
}

impl Syscall {
    pub fn new(number: usize, function: *const ()) -> Self {
        Self { number, function }
    }

    pub fn function_pointer(&self) -> *const () {
        self.function
    }

    pub fn number(&self) -> usize {self.number}
}


unsafe impl Send for Syscall {}
unsafe impl Sync for Syscall {}

pub struct NamingObject {
    handle: usize,
    object_type: ObjectType,
}

#[derive(Copy, Clone)]
pub enum ObjectType {
    File,
    Directory,
    Pipe,
}

impl NamingObject {
    pub fn new(handle: usize, object_type: ObjectType) -> Self {
        Self { handle, object_type }
    }

    pub fn handle(&self) -> usize {
        self.handle
    }
}

pub fn create_naming_capability(handle: usize, object_type: ObjectType, options: OpenOptions) -> Capability<NamingObject> {
    let mut flags = CapabilityFlags::empty();

    if options.contains(OpenOptions::READONLY) {
        flags |= CapabilityFlags::READ;
    }
    if options.contains(OpenOptions::READWRITE) {
        flags |= CapabilityFlags::READ | CapabilityFlags::WRITE;
    }
    if options.contains(OpenOptions::WRITEONLY) {
        flags |= CapabilityFlags::READ | CapabilityFlags::WRITE; //without read cant invoke capability TODO check if thats alright??
    }
    /*
    if options.contains(OpenOptions::SHARE) {
        flags |= CapabilityFlags::SHARE;
    }
    
     */

    Capability::new(NamingObject::new(handle, object_type), flags)
}
