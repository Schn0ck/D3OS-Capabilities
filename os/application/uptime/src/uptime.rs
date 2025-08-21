#![no_std]

extern crate alloc;

use alloc::vec;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use time::systime;
use capabilities::revoke;
use concurrent::thread;

#[unsafe(no_mangle)]
pub fn main() {
    /*
    let systime = systime();

    if systime.num_seconds() < 60 {
        println!("{}", systime.num_seconds());
    } else if systime.num_seconds() < 3600 {
        println!("{}:{:0>2}", systime.num_minutes(), systime.num_seconds() % 60);
    } else {
        let seconds = systime.num_seconds() - (systime.num_minutes() * 60);
        println!("{}:{:0>2}:{:0>2}", systime.num_hours(), systime.num_minutes() % 60, seconds);
    }
    
     */

    let id = thread::start_application("date", vec![]).expect("Failed to start date application").id();
    revoke(id, 13);
    println!("revoked");
}