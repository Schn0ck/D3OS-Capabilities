#![no_std]

extern crate alloc;

use alloc::vec::Vec;
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
use concurrent::thread::{current, sleep, Thread};

fn revoke_thread() {
    let Ok(file) = mkfifo("/revoke", OpenOptions::READWRITE | OpenOptions::CREATE | OpenOptions::SHARE, ROOT) else {
        panic!()
    };

    sleep(500);
    share_naming_object(10, file); // Assuming main thread has ID 10
    
    // Try to use the capability before it's revoked
    // let mut buf = [0u8];
    // let _read = read(file, &mut buf);
    // println!("Read value before revoke: {}", buf[0]);

    sleep(4000);
    
    // Revoke the capability

    revoke_naming_object(10, file); // Assuming main thread has ID 10


    // sleep(10000);

    loop {
        
    }
}

#[unsafe(no_mangle)]
pub fn main() {
    // Create the revoke thread
    let _revoke = thread::create(revoke_thread);


    println!("Revoke thread started with id: {}", current().unwrap().id() + 1);
    thread::start_application("revoketest2", Vec::new());
    thread::start_application("revoketest3", Vec::new());
}