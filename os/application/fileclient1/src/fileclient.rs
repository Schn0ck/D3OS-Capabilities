#![no_std]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use naming::shared_types::OpenOptions;
use naming::{mkfifo, read, write, ROOT};
use naming::{SHARED_PIPE};
use capabilities::share_naming_object;
#[allow(unused_imports)]
use runtime::*;
use syscall::return_vals::Errno;
use terminal::{print};

type FileHandle = usize;

pub struct FileClient {
    pipe_cap: usize,
}

impl FileClient {
    pub fn connect() -> Option<Self> {
        print!("---fileclient: connecting to file server...\n");

        let Ok(pipe) = mkfifo("/client1", OpenOptions::READWRITE | OpenOptions::SHARE, ROOT) else {
            print!("---fileclient: failed to create client pipe\n");
            return None;
        };

        print!("---fileclient: created client pipe with cap = {}\n", pipe);

        let mut thread_id = [1u8];
        let mut ack = [0u8; 1];


        read(SHARED_PIPE, &mut thread_id); //read server thread id from shared pipe //Todo not hardcoded
        print!("---fileclient: got thread id = {}\n", thread_id[0] as usize);
        share_naming_object(thread_id[0] as usize, pipe); //share client pipe with server

        read(pipe, &mut ack); //read ACK

        print!("---fileclient: got server response = {}\n", ack[0] as usize);

        Some(FileClient { pipe_cap: pipe })
    }

    pub fn write_file(&mut self, content: &[u8]) -> Result<FileHandle, Errno> {
        // Prepare command buffer: command byte + content length
        let mut cmd = [0u8; 9];
        cmd[0] = 1; // Write command

        // Store content length as little endian bytes
        let len = content.len() as usize;
        unsafe {
            *(cmd[1..9].as_mut_ptr() as *mut usize) = len.to_le();
        }

        // Send command and length
        write(self.pipe_cap, &cmd)?;

        // Send content
        write(self.pipe_cap, content)?;

        // Read back handle
        let mut handle_buf = [0u8; 8];
        read(self.pipe_cap, &mut handle_buf)?;

        Ok(unsafe { *(handle_buf.as_ptr() as *const usize) }.to_le())
    }

    pub fn read_file(&mut self, handle: FileHandle) -> Result<Vec<u8>, Errno> {
        // Prepare command buffer: command byte + file handle
        let mut cmd = [0u8; 9];
        cmd[0] = 2; // Read command

        unsafe {
            *(cmd[1..9].as_mut_ptr() as *mut usize) = handle.to_le();
        }

        // Send command and handle
        write(self.pipe_cap, &cmd)?;

        // Read size
        let mut size_buf = [0u8; 8];
        read(self.pipe_cap, &mut size_buf)?;

        let size = unsafe { *(size_buf.as_ptr() as *const usize) }.to_le() as usize;
        if size == 0 {
            return Err(Errno::ENOENT);
        }

        // Read content
        let mut content = vec![0u8; size];
        read(self.pipe_cap, &mut content)?;

        Ok(content)
    }
}

// Example usage
#[unsafe(no_mangle)]
pub fn main(){
    let Some(mut client) = FileClient::connect() else {
        print!("fileclient: failed to connect to file server\n");
        return;
    };
    //
    // // Write a file
    // let handle = client.write_file(b"Hello, File Server!")?;
    // print!("Wrote file with handle: {}\n", handle);
    //
    // // Read it back
    // let content = client.read_file(handle)?;
    // let content_str = core::str::from_utf8(&content).unwrap();
    // print!("Read back: {}\n", content_str);
    //
    // Ok(())
}