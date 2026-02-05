#[derive(Copy, Clone)]
pub struct Capability{
    handle: usize,
}

impl Capability {
    pub const fn new(handle: usize) -> Self {
            Self { handle }
    }

    pub fn handle(&self) -> usize {
        self.handle
    }
}
