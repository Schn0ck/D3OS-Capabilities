#![no_std]

extern crate alloc;

use core::ptr::null;
use concurrent::{process, thread};
use naming::{mkfifo, open, read, write, ROOT};
use naming::shared_types::OpenOptions;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use terminal::write::print;
use capabilities::revoke_naming_object;
use concurrent::thread::current;

#[unsafe(no_mangle)]
pub fn main() {
    let Ok(file) = mkfifo("/revoke", OpenOptions::READWRITE | OpenOptions::CREATE, ROOT) else {
        panic!()
    };
    
    let _write = write(file, &[1u8]);

    revoke_naming_object(current().unwrap().id(), file);
    
    let mut buf = [0u8];
    let _read = read(file, &mut buf);
    
    println!("{}", buf[0]); 
    
    
    // let process = process::current().unwrap();
    // let thread = thread::current().unwrap();
    // 
    // let pipe = mkfifo("/tmp/pipe", OpenOptions::READWRITE, ROOT).expect("Failed to create FIFO");
    // 
    // revoke(pipe).expect("Failed to revoke FIFO");
    // 
    // write(pipe, b"Hello, File Server!").expect("Failed to write to FIFO");
}