use core::arch::asm;
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
                    if cspace.receive_syscall_capability(syscall_cap.share(syscall_cap.get_permissions()), syscall_number){//Check for Share Flag happens here
                        return 0;
                    } 
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