#![no_std]

extern crate alloc;

use concurrent::{process, thread};
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};


#[unsafe(no_mangle)]
pub fn main() {
    todo!();
    // let process = process::current().unwrap();
    // let thread = thread::current().unwrap();
    // 
    // let pipe = mkfifo("/tmp/pipe", OpenOptions::READWRITE, ROOT).expect("Failed to create FIFO");
    // 
    // revoke(pipe).expect("Failed to revoke FIFO");
    // 
    // write(pipe, b"Hello, File Server!").expect("Failed to write to FIFO");
}