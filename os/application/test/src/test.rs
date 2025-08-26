#![no_std]

extern crate alloc;

use alloc::vec;
use capabilities::revoke;
use concurrent::{process, thread};
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};


#[unsafe(no_mangle)]
pub fn main() {
    let fileserver = thread::start_application("fileserver", vec![]).expect("Failed to start date application");
    
    //TODO send message via ipc to fileserver to create a file /tmp/testfile
    //TODO send message via ipc to fileserver to write to the file /tmp/testfile
    //TODO send message via ipc to fileserver to read from the file /tmp/testfile
    //TODO send message via ipc to fileserver to close the file /tmp/testfile
    
    
    /*
    let date = thread::start_application("date", vec![]).expect("Failed to start date application");
    thread::sleep(100);
    revoke(date.id(), 13);
    println!("revoked");
     */
}