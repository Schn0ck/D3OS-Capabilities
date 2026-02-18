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
use capabilities::{get_naming_len, revoke_naming_object, share_naming_object};
use capabilities::capability::Capability;
use concurrent::thread::{sleep, Thread, current};
#[unsafe(no_mangle)]
pub fn main() {
    let file = Capability::new(get_naming_len() - 1);
    let mut buf = [0u8; 4];

    sleep(1000);

    let res = write(file, "Hello".as_ref());
    if res.is_err() {
        println!("write error = {:?}", res);
    } else {
        println!("write successful");
    }

    //sleep(1000);

    let res = read(file, &mut buf);

    if res.is_err() {//todo doesnt print anything
        println!("read error = {:?}", res);
    } else {
        print!("read success:");
        println!("{:?}", &buf)
    }

    loop {
        println!("end");
    }
}