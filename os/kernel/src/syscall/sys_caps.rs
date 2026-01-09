use core::arch::asm;
use log::{error, info};
use crate::capabilities::capability::{Capability, CapabilityFlags};
use crate::scheduler;

/**
Share cap with same permissions
 */
pub extern "sysv64" fn sys_share_syscall_cap(thread_id: usize, syscall_number: usize) -> isize { //TODO handle the same way as share naming cap
    let cur_thread = scheduler().current_thread();
    let shared_cap =
        if let Some(sharer_cspace) = cur_thread.cspace.invoke(){
            if let Some(syscall_cap) = sharer_cspace.get_syscall_capability(syscall_number) {
                syscall_cap.share(syscall_cap.get_permissions())
            } else {
                error!(" sharing naming cap: naming cap not found in sharer cspace");
                None
            }
        } else {
            error!(" sharing naming cap: failed to invoke sharer cspace");
            None
        };

    if let Some(receiver_thread) = scheduler().thread(thread_id) {
        if let Some(mut cspace) = receiver_thread.cspace.invoke() {
            info!("     cspace found");
            if let Some(ref cap) = shared_cap {
                return cspace.receive_syscall_capability(shared_cap, syscall_number);
            }
        } else {
            error!(" receiver cspace not found")
        }
    }
    -5
}
pub extern "sysv64" fn sys_revoke_syscall_cap(thread_id: usize, syscall_number: usize) -> isize {
    if let Some(thread) = scheduler().thread(thread_id){
        if let Some(mut cspace) = thread.cspace.invoke(){
            cspace.revoke_syscall_capability(syscall_number);
            return 0;
        }
    }
    -5
}

pub extern "sysv64" fn sys_share_naming_cap(thread_id: usize, naming_object_number: usize) -> isize {
    let cur_thread = scheduler().current_thread();

    
    // Scope the first lock so it's dropped before we try to acquire the second one
    let shared_cap = 
        if let Some(sharer_cspace) = cur_thread.cspace.invoke(){
            if let Some(naming_cap) = sharer_cspace.get_naming_capability(naming_object_number) {
                naming_cap.share(naming_cap.get_permissions())
            } else {
                error!(" sharing naming cap: naming cap not found in sharer cspace");
                None
            }
        } else {
            error!(" sharing naming cap: failed to invoke sharer cspace");
            None
    };

    if let Some(receiver_thread) = scheduler().thread(thread_id) {
        if let Some(mut cspace) = receiver_thread.cspace.invoke() {
            info!("     cspace found");
            if let Some(ref cap) = shared_cap {
                return cspace.receive_naming_capability(shared_cap);
            }
        } else {
            error!(" receiver cspace not found")
        }
    }
    -5
}

//TODO revoke/delete