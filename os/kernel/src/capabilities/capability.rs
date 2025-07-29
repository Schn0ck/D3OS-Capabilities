use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct CapabilityFlags: u32 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
        const EXECUTE = 0b00000100;
        const SHARE = 0b00001000;
        const TRANSFER = 0b00010000;
    }
}

pub struct Capability<T> {
    obj: Option<T>,
    flags: CapabilityFlags,
}

impl<T> Capability<T> {
    fn invoke_mut(&mut self) -> Option<&mut T> {
        if Option::is_some(&self.obj) {
            todo!() // Read-Write-Delete?
        }
        None
    }
    
    fn invoke(&mut self) -> Option< T> {
        if Option::is_some(&self.obj) && has_permissions(&self.obj, CapabilityFlags::READ) {
            todo!() //Read-Only?
        }
        None
    }
}

fn has_permissions<T>(p0: T, p1: CapabilityFlags) -> bool {
    todo!()
}