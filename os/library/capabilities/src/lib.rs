#![no_std]

use syscall::{syscall, SystemCall};
use core::result::Result::{Err, Ok};

pub fn share(thread_id: usize, syscall_num: usize) -> bool{
    let res = syscall(SystemCall::ShareSyscallCap, &[thread_id, syscall_num]);
    match res {
        Ok(b) => b == 0,
        Err(_) => core::panic!("Syscall: Share Syscall Cap failed."),
    }
}
pub fn revoke(thread_id: usize, syscall_num: usize) {
    let res = syscall(SystemCall::RevokeSyscallCap, &[thread_id, syscall_num]);
}