/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: sys_naming                                                      ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: All system calls for the naming service.                        ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Michael Schoettner, 25.08.2025, HHU                             ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use alloc::slice;
use alloc::string::{String, ToString};
use core::ptr::slice_from_raw_parts;
use core::str::from_utf8;
use core::mem;
use log::{info,warn};
use naming::shared_types::{OpenOptions, SeekOrigin, RawDirent};
use syscall::return_vals::{self, Errno};
use num_enum::FromPrimitive;
use crate::capabilities::capability::Capability;
use crate::capabilities::capability_objects::naming_object::NamingObject;
use crate::naming::api;
use crate::scheduler;
use crate::syscall::syscall_dispatcher::init;
/*pub unsafe extern "sysv64" fn sys_open(path: *const u8, flag_bits: usize) -> isize {
    let flags = OpenOptions::from_bits(flag_bits).unwrap();
    return_vals::convert_syscall_result_to_ret_code(api::open(&unsafe { ptr_to_string(path).unwrap() }, flags))
}*/

pub unsafe extern "sysv64" fn sys_root(flag_bits: usize) -> isize {
    let flags = OpenOptions::from_bits(flag_bits).unwrap();

    return match api::root() {
        Ok(cap) => {
            // Store capability in current thread's CSpace
            if let Some(mut cspace) = scheduler().current_thread().cspace.invoke() {
                // Store the capability and return its handle
                // Implementation depends on your CSpace management
                return cspace.receive_root_naming_capability(Some(cap));;
            }
            Errno::EACCES as isize
        }
        Err(errno) => errno as isize
    }
}

pub unsafe extern "sysv64" fn sys_open(path: *const u8, flag_bits: usize, cap_handle: usize) -> isize {
    let current_thread = scheduler().current_thread();
    let flags = OpenOptions::from_bits(flag_bits).unwrap();
    let path = unsafe { ptr_to_string(path).unwrap() };
    let cspace = current_thread.cspace.invoke().unwrap();
    let naming_cap = cspace.get_naming_capability(cap_handle);

    if let Some(cap) = naming_cap {
        match api::open(&*path, flags, &cap) {
            Ok(cap) => {
                // Store capability in current thread's CSpace
                if let Some(mut cspace) = scheduler().current_thread().cspace.invoke() {
                    // Store the capability and return its handle
                    // Implementation depends on your CSpace management
                    let handle = cspace.receive_naming_capability(Some(cap));
                    return handle;
                }
                return Errno::EACCES as isize;
            }
            Err(errno) => return errno as isize
        }
    }
    
    Errno::EACCES as isize
}


pub unsafe extern "sysv64" fn sys_read(cap_handle: usize, buffer: *mut u8, buffer_length: usize) -> isize {
    if buffer.is_null() || buffer_length == 0 {
        return Errno::EINVAL as isize;
    }

    let current_thread = scheduler().current_thread();
    
    // Get the naming capability while holding the cspace lock
    let cspace = current_thread.cspace.invoke().unwrap();
    let naming_cap = cspace.get_naming_capability(cap_handle);

    // Now we can safely drop the cspace lock and proceed with the read operation
    if let Some(cap) = naming_cap {
        let buf = unsafe { slice::from_raw_parts_mut(buffer, buffer_length) };
        return return_vals::convert_syscall_result_to_ret_code(api::read(&cap, buf));
    }

    Errno::EACCES as isize
}


/*pub unsafe extern "sysv64" fn sys_write(fh: usize, buffer: *const u8, buffer_length: usize) -> isize {
    if buffer.is_null() || buffer_length == 0 {
        return Errno::EINVAL as isize;
    }
    let buf: &[u8];
    unsafe {
        buf = slice::from_raw_parts(buffer, buffer_length);
    }
    return_vals::convert_syscall_result_to_ret_code(api::write(fh, buf))
}*/

pub unsafe extern "sysv64" fn sys_write(cap_handle: usize, buffer: *mut u8, buffer_length: usize) -> isize { //todo pagefault when invalid cap
    if buffer.is_null() || buffer_length == 0 {
        return Errno::EINVAL as isize;
    }

    let current_thread = scheduler().current_thread();
    let cspace = current_thread.cspace.invoke().unwrap();
    let naming_cap = cspace.get_naming_capability(cap_handle);

    // Now we can safely drop the cspace lock and proceed with the read operation
    if let Some(cap) = naming_cap {
        let buf = unsafe { slice::from_raw_parts_mut(buffer, buffer_length) };
        return return_vals::convert_syscall_result_to_ret_code(api::write(&cap, buf));
    }

    Errno::EACCES as isize
}

pub extern "sysv64" fn sys_seek(cap_handle: usize, offset: usize, origin: usize) -> isize {
    let current_thread = scheduler().current_thread();
    let cspace = current_thread.cspace.invoke().unwrap();
    let naming_cap = cspace.get_naming_capability(cap_handle);

    if let Some(cap) = naming_cap {
        return return_vals::convert_syscall_result_to_ret_code(api::seek(&cap, offset, SeekOrigin::from_primitive(origin)));
    }

    Errno::EACCES as isize
}

/*pub extern "sysv64" fn sys_close(fh: usize) -> isize {
    return_vals::convert_syscall_result_to_ret_code(api::close(fh))
}*/

pub extern "sysv64" fn sys_close(cap_handle: usize) -> isize {
    let current_thread = scheduler().current_thread();
    let cspace = current_thread.cspace.invoke().unwrap();
    let naming_cap = cspace.get_naming_capability(cap_handle);

    if let Some(cap) = naming_cap {
        return return_vals::convert_syscall_result_to_ret_code(api::close(&cap));
    }
    Errno::EACCES as isize
}

pub unsafe extern "sysv64" fn sys_mkdir(path: *const u8, cap_to_dir: Capability<NamingObject>, cap_handle: usize) -> isize {
    let path = unsafe { ptr_to_string(path).unwrap() };
    match api::mkdir(&*path, cap_to_dir, cap_handle) {
        Ok(cap) => {
            // Store capability in current thread's CSpace
            let handle = {
                if let Some(mut cspace) = scheduler().current_thread().cspace.invoke() {
                // Store the capability and return its handle
                // Implementation depends on your CSpace management
                cspace.receive_naming_capability(Some(cap))
                } else {
                    Errno::EACCES as isize
                }
            };
            handle
        }
        Err(errno) => errno as isize
    }
}

pub unsafe extern "sysv64" fn sys_touch(path: *const u8, cap_handle: usize) -> isize {//TODO why handle, not cap?
    let current_thread = scheduler().current_thread();
    let path = unsafe { ptr_to_string(path).unwrap() };
    let cspace = current_thread.cspace.invoke().unwrap();
    let naming_cap = cspace.get_naming_capability(cap_handle);

    if let Some(cap) = naming_cap {
        match api::touch(&*path, &cap) {
            Ok(cap) => {
                // Store capability in current thread's CSpace
                if let Some(mut cspace) = scheduler().current_thread().cspace.invoke() {
                    // Store the capability and return its handle
                    // Implementation depends on your CSpace management
                    let handle = cspace.receive_naming_capability(Some(cap));
                    return handle;
                }
            },
            Err(_) => return Errno::EINVAL as isize,
        }
    }
    Errno::EACCES as isize
}

pub unsafe extern "sysv64" fn sys_mkfifo(path: *const u8, flag_bits: usize, dir_cap_handle: usize) -> isize {
    let current_thread = scheduler().current_thread();
    let flags = OpenOptions::from_bits(flag_bits).unwrap();
    let path = unsafe { ptr_to_string(path).unwrap() };
    
    info!("sys_mkfifo called with path: {}, flags: {:?}, dir_cap_handle: {}", path, flags, dir_cap_handle);
    
    // Get the capability and release the cspace lock before api call
    let mut cspace = current_thread.cspace.invoke().unwrap();
    let dir_cap = cspace.get_naming_capability(dir_cap_handle);

    // Now make the api call with no locks held
    if let Some(dir_cap) = dir_cap {
        match api::mkfifo(&*path, flags, &dir_cap) {
            Ok(new_cap) => {
                // Reacquire the lock to store the new capability
                info!("mkfifo succeeded, storing new capability");
                return cspace.receive_naming_capability(Some(new_cap));
            }
            Err(errno) => {
                warn!("mkfifo failed");
                return errno as isize;
            }
        }
    }

    Errno::EACCES as isize
}

    /// Convert a raw pointer resulting from a CString to a UTF-8 String
pub(super) unsafe fn ptr_to_string(ptr: *const u8) -> Result<String, Errno> {
    if ptr.is_null() {
        return Err(Errno::EBADSTR);
    }

    let mut len = 0;
    // Find the null terminator to determine the length
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }

    let path = from_utf8(unsafe { slice_from_raw_parts(ptr, len).as_ref().unwrap() });
    match path {
        Ok(path_str) => Ok(path_str.to_string()),
        Err(_) => Err(Errno::EBADSTR),
    }
}

pub unsafe extern "sysv64" fn sys_readdir(fh: usize, buffer: *mut u8, buffer_length: usize) -> isize {
    if buffer.is_null() || buffer_length == 0 || buffer_length <  mem::size_of::<RawDirent>() {
        return Errno::EINVAL as isize;
    }
    let dentry_ptr = buffer as *mut RawDirent;
    let dentry = unsafe { dentry_ptr.as_mut() };
    return_vals::convert_syscall_result_to_ret_code(api::readdir(fh, dentry))
}


pub unsafe extern "sysv64" fn sys_cwd(buffer: *mut u8, buffer_length: usize) -> isize {
    if buffer.is_null() || buffer_length == 0 {
        return Errno::EINVAL as isize;
    }
    let buf: &mut[u8];
    unsafe {
        buf = slice::from_raw_parts_mut(buffer, buffer_length);
    }
    return_vals::convert_syscall_result_to_ret_code(api::cwd(buf))
}

pub unsafe extern "sysv64" fn sys_cd(path: *const u8) -> isize {
    return_vals::convert_syscall_result_to_ret_code(api::cd(&unsafe {ptr_to_string(path)}.unwrap()))
}