#![no_std]

extern crate alloc;

#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::vec;
use spin::Mutex;
use naming::{close, open, read, write};
use syscall::return_vals::Errno;
use concurrent::thread;

#[unsafe(no_mangle)]
pub unsafe fn main() {
    println!("FileServer: service starting");

    // Create our file server instance
    let server = FileServer::new();
    println!("FileServer: initialized");

    // Main service loop
    loop {
        // Handle any pending operations
        if let Some(msg) = check_for_messages() {
            match server.handle_message(msg) {
                Ok(response) => println!("FileServer: operation successful"),
                Err(e) => println!("FileServer: operation failed"),
            }
        }

        thread::switch();
    }
}


// Placeholder until IPC is implemented
fn check_for_messages() -> Option<FSMessage> {
    use core::time::Duration;

    let random = Duration::from_nanos(0).as_nanos() as u32;
    match random % 4 {
        0 => Some(FSMessage::Open {
            path: "/tmp/testfile".to_string(),
            flags: FileFlags(FileFlags::READ | FileFlags::WRITE),
        }),
        1 => Some(FSMessage::Read {
            handle: 1,
            len: 1024,
        }),
        2 => Some(FSMessage::Write {
            handle: 1,
            data: Vec::new(),
        }),
        3 => Some(FSMessage::Close {
            handle: 1
        }),
        _ => None
    }
}

// File server message types
#[derive(Clone)]
pub enum FSMessage {
    Open {
        path: String,
        flags: FileFlags,
    },
    Read {
        handle: usize,
        len: usize,
    },
    Write {
        handle: usize,
        data: Vec<u8>,
    },
    Close {
        handle: usize,
    },
    CreatePipe {
        name: String,
        flags: FileFlags,
    },
    ConnectPipe {
        name: String,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub struct FileFlags(u32);

impl FileFlags {
    const READ: u32 = 0b0001;
    const WRITE: u32 = 0b0010;
    const CREATE: u32 = 0b0100;
    const PIPE: u32 = 0b1000;
    
    pub fn new(bits: u32) -> Self {
        Self(bits)
    }
    
    pub fn can_read(&self) -> bool { self.0 & Self::READ != 0 }
    pub fn can_write(&self) -> bool { self.0 & Self::WRITE != 0 }
    pub fn is_pipe(&self) -> bool { self.0 & Self::PIPE != 0 }
}

struct FileHandle {
    path: String,
    flags: FileFlags,
    pipe_partner: Option<usize>, // For pipes: handle of the other end
}

pub struct FileServer {
    handles: Mutex<BTreeMap<usize, FileHandle>>,
    pipes: Mutex<BTreeMap<String, Vec<usize>>>, // pipe name -> handles
    next_handle: Mutex<usize>,
}

impl FileServer {
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(BTreeMap::new()),
            pipes: Mutex::new(BTreeMap::new()),
            next_handle: Mutex::new(1),
        }
    }

    fn allocate_handle(&self) -> usize {
        let mut handle = self.next_handle.lock();
        let current = *handle;
        *handle += 1;
        current
    }

    pub fn handle_message(&self, msg: FSMessage) -> Result<Vec<u8>, Errno> {
        match msg {
            FSMessage::Open { path, flags } => self.handle_open(path, flags),
            FSMessage::Read { handle, len } => self.handle_read(handle, len),
            FSMessage::Write { handle, data } => self.handle_write(handle, data),
            FSMessage::Close { handle } => self.handle_close(handle),
            FSMessage::CreatePipe { name, flags } => self.handle_create_pipe(name, flags),
            FSMessage::ConnectPipe { name } => self.handle_connect_pipe(name),
        }
    }

    fn handle_open(&self, path: String, flags: FileFlags) -> Result<Vec<u8>, Errno> {
        // Convert our flags to naming service flags
        let ns_flags = {
            let mut opts = naming::shared_types::OpenOptions::empty();
            if flags.can_read() { opts |= naming::shared_types::OpenOptions::READONLY; }
            if flags.can_write() { opts |= naming::shared_types::OpenOptions::READWRITE; }
            if flags.0 & FileFlags::CREATE != 0 { opts |= naming::shared_types::OpenOptions::CREATE; }
            opts
        };

        // Try to open via naming service
        match open(&path, ns_flags) {
            Ok(ns_handle) => {
                let handle = self.allocate_handle();
                let mut handles = self.handles.lock();
                handles.insert(handle, FileHandle {
                    path,
                    flags,
                    pipe_partner: None,
                });
                Ok(handle.to_ne_bytes().to_vec())
            }
            Err(e) => Err(e),
        }
    }

    fn handle_read(&self, handle: usize, len: usize) -> Result<Vec<u8>, Errno> {
        let handles = self.handles.lock();
        if let Some(file_handle) = handles.get(&handle) {
            if !file_handle.flags.can_read() {
                return Err(Errno::EACCES);
            }

            if file_handle.flags.is_pipe() {
                // Handle pipe reading
                if let Some(partner) = file_handle.pipe_partner {
                    // Read from pipe partner's buffer
                    // This is where you'd implement the pipe buffer reading
                    Ok(Vec::new()) // Placeholder
                } else {
                    Err(Errno::EUNKN)  // No partner connected
                }
            } else {
                // Regular file reading
                let mut buffer = vec![0u8; len];
                match read(handle, &mut buffer) {
                    Ok(bytes_read) => Ok(buffer[..bytes_read].to_vec()),
                    Err(e) => Err(e),
                }
            }
        } else {
            Err(Errno::EBADF)
        }
    }

    fn handle_write(&self, handle: usize, data: Vec<u8>) -> Result<Vec<u8>, Errno> {
        let handles = self.handles.lock();
        if let Some(file_handle) = handles.get(&handle) {
            if !file_handle.flags.can_write() {
                return Err(Errno::EACCES);
            }

            if file_handle.flags.is_pipe() {
                // Handle pipe writing
                if let Some(partner) = file_handle.pipe_partner {
                    // Write to pipe partner's buffer
                    // This is where you'd implement the pipe buffer writing
                    Ok(data.len().to_ne_bytes().to_vec())
                } else {
                    Err(Errno::EUNKN) // No partner connected
                }
            } else {
                // Regular file writing
                match write(handle, &data) {
                    Ok(bytes_written) => Ok(bytes_written.to_ne_bytes().to_vec()),
                    Err(e) => Err(e),
                }
            }
        } else {
            Err(Errno::EBADF)
        }
    }

    fn handle_create_pipe(&self, name: String, flags: FileFlags) -> Result<Vec<u8>, Errno> {
        let handle = self.allocate_handle();
        
        let mut pipes = self.pipes.lock();
        let mut handles = self.handles.lock();
        
        // Create new pipe entry
        pipes.insert(name.clone(), vec![handle]);
        
        // Create handle
        handles.insert(handle, FileHandle {
            path: name,
            flags: FileFlags::new(flags.0 | FileFlags::PIPE),
            pipe_partner: None,
        });

        Ok(handle.to_ne_bytes().to_vec())
    }

    fn handle_connect_pipe(&self, name: String) -> Result<Vec<u8>, Errno> {
        let mut pipes = self.pipes.lock();
        
        if let Some(existing_handles) = pipes.get_mut(&name) {
            if existing_handles.len() == 1 {
                // We can connect to this pipe
                let new_handle = self.allocate_handle();
                let partner_handle = existing_handles[0];
                
                let mut handles = self.handles.lock();
                
                // Update partner's pipe_partner
                if let Some(partner) = handles.get_mut(&partner_handle) {
                    partner.pipe_partner = Some(new_handle);
                }
                
                // Create new handle
                handles.insert(new_handle, FileHandle {
                    path: name,
                    flags: FileFlags::new(FileFlags::READ | FileFlags::WRITE | FileFlags::PIPE),
                    pipe_partner: Some(partner_handle),
                });
                
                existing_handles.push(new_handle);
                
                Ok(new_handle.to_ne_bytes().to_vec())
            } else {
                Err(Errno::EEXIST) // Pipe already has two ends connected
            }
        } else {
            Err(Errno::ENOENT) // Pipe doesn't exist
        }
    }

    fn handle_close(&self, handle: usize) -> Result<Vec<u8>, Errno> {
        let mut handles = self.handles.lock();
        if let Some(file_handle) = handles.remove(&handle) {
            if file_handle.flags.is_pipe() {
                // Clean up pipe connections
                let mut pipes = self.pipes.lock();
                if let Some(pipe_handles) = pipes.get_mut(&file_handle.path) {
                    pipe_handles.retain(|&h| h != handle);
                    if pipe_handles.is_empty() {
                        pipes.remove(&file_handle.path);
                    }
                }
            }
            close(handle).map(|_| Vec::new())
        } else {
            Err(Errno::EBADF)
        }
    }
}