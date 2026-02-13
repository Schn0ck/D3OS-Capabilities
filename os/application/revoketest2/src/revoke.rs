#![no_std]

extern crate alloc;

use alloc::string::String;
use core::ptr::null;
use concurrent::{process, thread};
use naming::{mkfifo, open, read, write, ROOT};
use naming::shared_types::OpenOptions;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use terminal::write::print;
use capabilities::{revoke_naming_object, share_naming_object};
use capabilities::capability::Capability;
use concurrent::thread::{sleep, Thread, current};
#[unsafe(no_mangle)]
pub fn main() {
    let file = Capability::new(2);

    sleep(10000);

    write(file, "Hello World!\n".as_ref());

    sleep(10000);

    let mut buf = [0u8; 12];
    read(file, &mut buf);

    println!("{:?}", String::from_utf8_lossy(&buf[..]))
}