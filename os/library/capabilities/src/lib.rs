#![no_std]
pub mod capability;

use syscall::{syscall, SystemCall};
use core::result::Result::{Err, Ok};
use terminal::{print, println};
use crate::capability::Capability;

pub fn share_syscall(thread_id: usize, syscall_num: usize) -> bool{
    let res = syscall(SystemCall::ShareSyscallCap, &[thread_id, syscall_num]);
    match res {
        Ok(b) => b == 0,
        Err(_) => panic!("Syscall: Share Syscall Cap failed."), //TODO: no panic necessary if no permissions
    }
}
pub fn revoke(thread_id: usize, syscall_num: usize) -> isize{
    let res = syscall(SystemCall::RevokeSyscallCap, &[thread_id, syscall_num]);
    match res {
        Ok(b) => b as isize,
        Err(e) => {
            println!("Syscall: Share Naming Cap failed with error {}", e as isize);
            e as isize
        },
    }
}
pub fn share_naming_object(thread_id: usize, cap: Capability) -> isize{ //todo dont share which handle -> tells caller how many caps
    let res = syscall(SystemCall::ShareNamingCap, &[thread_id, cap.handle()]);
    match res {
        Ok(b) => b as isize,
        Err(e) => {
            println!("Syscall: Share Naming Cap failed with error {}", e as isize);
            e as isize
        },
    }
}

pub fn get_naming_len() -> usize{
    let res = syscall(SystemCall::NamingLen, &[]);
    match res {
        Ok(b) => b,
        Err(e) => {
            print!("Syscall: Get Naming Len failed with error {}", e as isize);
            return 0;
        },
    }
}