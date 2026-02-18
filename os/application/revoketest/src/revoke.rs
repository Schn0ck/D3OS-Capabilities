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
    sleep(100);
    let Ok(file) = mkfifo("/revoke", OpenOptions::READWRITE | OpenOptions::CREATE, ROOT) else {
        panic!()
    };

    // Wait a bit to ensure the main thread has time to share the capability
    sleep(10000);
    
    // Try to use the capability before it's revoked
    // let mut buf = [0u8];
    // let _read = read(file, &mut buf);
    // println!("Read value before revoke: {}", buf[0]);
    
    // Revoke the capability
    //revoke_naming_object(9, file); // Assuming main thread has ID 1
    
    //println!("Capability revoked");

    loop {
        
    }
}

#[unsafe(no_mangle)]
pub fn main() {
    // Create the revoke thread
    let _revoke = thread::create(revoke_thread);


    println!("Revoke thread started with id: {}", current().unwrap().id() + 1);
    thread::start_application("revoketest2", Vec::new());
}