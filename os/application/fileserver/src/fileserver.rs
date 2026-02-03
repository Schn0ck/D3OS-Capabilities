#![no_std]

extern crate alloc;

use terminal::print;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::result::Result::Ok;
use core::result::Result;
use core::option::Option::*;
use naming::{mkfifo, read, write, ROOT, SHARED_PIPE};
use naming::shared_types::OpenOptions;
use syscall::return_vals::Errno;

#[allow(unused_imports)]
use runtime::*;
use terminal::{println};

type FileHandle = usize;

enum Command {
    Write { content: Vec<u8> },
    Read { handle: FileHandle },
}

enum Response {
    Handle(FileHandle),
    Content(Vec<u8>),
    Error,
}

pub struct FileServer {
    files: BTreeMap<FileHandle, Vec<u8>>,
    next_handle: usize,
    command_pipe_handle: usize,
}

impl FileServer {
    pub(crate) fn new(pipe_handle: usize) -> Result<Self, Errno> {
        // Create command pipe with read/write/share permissions
        write(pipe_handle, &[0u8])?;

        Ok(Self {
            files: BTreeMap::new(),
            next_handle: 1,
            command_pipe_handle: pipe_handle,
        })
    }

    pub(crate) fn run(&mut self, pipe_handle: usize) -> Result<(), Errno> {
        let mut cmd_buf = [0u8; 9]; // 1 byte command + 8 bytes data

        loop {
            // Read command
            read(pipe_handle, &mut cmd_buf)?;

            match cmd_buf[0] {
                // Write command
                1 => {
                    let mut content = Vec::new();
                    let size = unsafe {
                        let ptr = cmd_buf[1..9].as_ptr();
                        // Force unaligned read if necessary
                        core::ptr::read_unaligned(ptr as *const u64)
                    }.to_le() as usize;


                    let mut data = vec![0u8; size];
                    read(self.command_pipe_handle, &mut data)?;
                    content.extend_from_slice(&data);

                    let handle = self.next_handle + 1;
                    self.files.insert(handle, content);

                    // Send back handle
                    let response = handle.to_le_bytes();
                    write(self.command_pipe_handle, &response)?;
                }

                // Read command
                2 => {
                    let handle = unsafe {
                        let ptr = cmd_buf[1..9].as_ptr();
                        // Force unaligned read if necessary
                        core::ptr::read_unaligned(ptr as *const u64)
                    }.to_le() as usize;

                    if let Some(content) = self.files.get(&handle) {
                        // Send size first
                        let size = content.len() as u64;
                        write(self.command_pipe_handle, &size.to_le_bytes())?;

                        // Then send content
                        write(self.command_pipe_handle, content)?;
                    } else {
                        // Send 0 size to indicate error
                        write(self.command_pipe_handle, &0u64.to_le_bytes())?;
                    }
                }

                _ => {} // Invalid command
            }
        }

    }
}

fn server_thread() {
    let naming_len = capabilities::get_naming_len(); //todo: this wont work if multiple naming caps are added quickly
    let mut server = FileServer::new(naming_len)
        .expect("Failed to create file server");
    println!("File server thread started for pipe at {}", naming_len);
    if let Err(e) = server.run(naming_len) {
        println!("File server error: {:?}", e);
    }

}

fn monitor_thread() {
    let mut naming_len = 2;

    loop {
        write(SHARED_PIPE, &[concurrent::thread::current().unwrap().id() as u8]);

        if capabilities::get_naming_len() > naming_len {
            //write cap_num into pipe so server thread knows which naming cap to use

            //create server thread
            let _ = concurrent::thread::create(server_thread);

            naming_len += 1;
        }

        concurrent::thread::sleep(1000);
    }
}

#[unsafe(no_mangle)]
pub fn main() -> () {
    // Create the monitoring thread
    let _ = concurrent::thread::create(monitor_thread);
    println!("  Started file server!")
}