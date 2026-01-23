#![no_std]

use syscall::{syscall, SystemCall};
use core::result::Result::{Err, Ok};
use terminal::{print, println};

pub fn share_syscall(thread_id: usize, syscall_num: usize) -> bool{
    let res = syscall(SystemCall::ShareSyscallCap, &[thread_id, syscall_num]);
    match res {
        Ok(b) => b == 0,
        Err(_) => panic!("Syscall: Share Syscall Cap failed."), //TODO: no panic necessary if no permissions
    }
}
pub fn revoke(thread_id: usize, syscall_num: usize) {
    let res = syscall(SystemCall::RevokeSyscallCap, &[thread_id, syscall_num]);
}
pub fn share_naming_object(thread_id: usize, naming_object_number: usize) -> usize{
    let res = syscall(SystemCall::ShareNamingCap, &[thread_id, naming_object_number]);
    match res {
        Ok(b) => b,
        Err(e) => {
            print!("Syscall: Share Naming Cap failed with error {}", e as isize);
            return 1;
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