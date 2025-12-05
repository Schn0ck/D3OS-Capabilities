use core::arch::asm;
use log::{error, info};
use crate::capabilities::capability::CapabilityFlags;
use crate::scheduler;

/**
Share cap with same permissions
 */
pub extern "sysv64" fn sys_share_syscall_cap(thread_id: usize, syscall_number: usize) -> isize {
    // Get current thread's CSpace through scheduler
    if let Some(sharer_cspace) = scheduler().current_thread().cspace.invoke() {
        if let Some(syscall_cap) = sharer_cspace.get_syscall_capability(syscall_number) {
            if let Some(receiver_thread) = scheduler().thread(thread_id){
                if let Some(mut cspace) = receiver_thread.cspace.invoke(){
                    return cspace.receive_syscall_capability(syscall_cap.share(syscall_cap.get_permissions()), syscall_number) //returns syscall number on success, -1 on failure
                }
            }
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

pub extern "sysv64" fn sys_share_naming_cap(thread_id: usize, naming_object_number: usize) -> isize { //TODO variable permissions
    // Get current thread's CSpace through scheduler
    info!("Sharing naming cap with number {} to thread {}", naming_object_number, thread_id);
    if let Some(sharer_cspace) = scheduler().current_thread().cspace.invoke() {
        info!("     sharer cspace found");
        if let Some(naming_cap) = sharer_cspace.get_naming_capability(naming_object_number) {
            info!("     naming cap found");
            if let Some(receiver_thread) = scheduler().thread(thread_id){
                info!("     receiver thread found, id {}", receiver_thread.id()); //TODO FAILS IN THE INFO!
                //info!("     receiver cspace is locked: {}", receiver_thread.cspace.is_locked());
                if let Some(mut cspace) = receiver_thread.cspace.invoke(){ //todo fails here!!! Invoking twice????
                    info!("     cspace found"); 
                    return cspace.receive_naming_capability(naming_cap.share(naming_cap.get_permissions())) //returns naming number on success, -1 on failure
                }
                else { 
                    error!(" receiver cspace not found")
                }
            }
        }
    }
    -5
}

//TODO revoke/delete