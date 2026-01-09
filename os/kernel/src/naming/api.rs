/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: api                                                             ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Public functions of the naming service:                                 ║
   ║   - init   init ns, called once                                         ║
   ║   - open   open a named object                                          ║
   ║   - read   read bytes from an open object                               ║
   ║   - write  write bytes into an open object                              ║
   ║   - seek   set file pointer (for files)                                 ║
   ║   - mkdir  create a directory                                           ║
   ║   - touch  create a file                                                ║
   ║   - mkfifo create a named pipe                                          ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Michael Schoettner, Univ. Duesseldorf, 25.8.2025                ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Pointer;
use core::sync::atomic::Ordering;
use log::{error, info, warn};
use spin::{Mutex, Once};

use super::lookup;
use super::open_objects;
use super::stat::Mode;
use super::tmpfs;
use super::traits::{FileSystem, NamedObject};

use crate::initrd;
use naming::shared_types::{OpenOptions, RawDirent, SeekOrigin};
use syscall::return_vals::Errno;
use crate::capabilities::capability::Capability;
use crate::capabilities::capability_objects::naming_object::{create_naming_capability, NamingObject, ObjectType};
use crate::syscall::sys_vmem::init_fb_info;

// root of naming service
pub(crate) static ROOT: Once<Arc<dyn FileSystem>> = Once::new();

// current working directory
static CWD: Mutex<String> = Mutex::new(String::new());

/// Initialize the naming service (must be called once before using it).
pub fn init() {
    // Initialize ROOT with TmpFs
    ROOT.call_once(|| {
        let tmpfs = tmpfs::TmpFs::new();

        for entry in initrd().entries() {
            let res = tmpfs.create_static_file(entry.filename().as_str().unwrap(), entry.data());
            if res.is_err() {
                warn!("Failed to create static file in tmpfs: {}", entry.filename().as_str().unwrap());
            }
        }

        Arc::new(tmpfs)
    });
    //open_objects::open_object_table_init();
    let mut cwd = CWD.lock();
    *cwd = "/".to_string();
    info!("naming service initialized");
    //    test::running_tests();
}

pub(crate) fn root() -> Result<Capability<NamingObject>, Errno> { //Every threat can access Root dir
    match open_objects::open("/", OpenOptions::all()){
        Ok(root) => {
            Ok(create_naming_capability(root, OpenOptions::all(), None))
        },
        Err(e) => {
            error!("root not found");
            Err(e)
        },
    }
}

/// Open/create a named object referenced by `path` using the given `flags`. \
/// Returns `Ok(object_handle)` or `Err`.
pub fn open(path: &str, flags: OpenOptions, cap_to_dir: &Capability<NamingObject>) -> Result<Capability<NamingObject>, Errno> {
    // avoid "opening" a file twice (only the creator receives capability and then needs to share it) 
    let result = lookup::lookup_named_object(path);
    if result.is_ok() {
        return Err(Errno::EEXIST);
    }

    // Try to open the object
    open_object(path, flags, cap_to_dir)
}

/// Write all bytes from the given `buffer` into the named object referenced by `object_handle`. \
/// Returns `Ok(number of bytes written)` or `Err`.
/*
pub fn write(object_handle: usize, buffer: &[u8]) -> Result<usize, Errno> {
    open_objects::write(object_handle, buffer)
}
 */

pub fn write(cap: &Capability<NamingObject>, buffer: &mut [u8]) -> Result<usize, Errno> {
    if let Some(naming_obj) = cap.invoke() {
        if naming_obj.named_object.is_file() {
            // Make `opened_object` mutable here
            return naming_obj.named_object.as_file().and_then(|file| {
                let pos = naming_obj.position.load(Ordering::SeqCst);
                let bytes_written = file.write(buffer, pos, naming_obj.access_rights)?;
                naming_obj.position.store(pos + bytes_written, Ordering::SeqCst);
                Ok(bytes_written) // Return the bytes written
            });
        }
        if naming_obj.named_object.is_pipe() {
            // Make `opened_object` mutable here
            return naming_obj.named_object.as_pipe().and_then(|pipe| {
                let bytes_written = pipe.write(buffer, 0, naming_obj.access_rights)?;
                info!("pipe written: {}", bytes_written);
                Ok(bytes_written) // Return the bytes written
            });
        }
        Err(Errno::ENOTSUP)
    } else {
        info!("could not invoke capability");
        Err(Errno::EACCES)
    }
}

/// Read from the named object referenced by `object_handle` into the given `buffer`. \
/// Returns `Ok(number of bytes read)` or `Err`.
/*
pub fn read(object_handle: usize, buffer: &mut [u8]) -> Result<usize, Errno> {
    open_objects::read(object_handle, buffer)
}
 */

pub fn read(cap: &Capability<NamingObject>, buffer: &mut [u8]) -> Result<usize, Errno> {
    if let Some(naming_obj) = cap.invoke() {
        if naming_obj.named_object.is_file() {
            // Make `opened_object` mutable here
            return naming_obj.named_object.as_file().and_then(|file| {
                let pos = naming_obj.position.load(Ordering::SeqCst);
                let bytes_read = file.read(buffer, pos, naming_obj.access_rights)?;
                naming_obj.position.store(pos + bytes_read, Ordering::SeqCst);
                Ok(bytes_read) // Return the bytes read
            });
        }
        if naming_obj.named_object.is_pipe() {
            // Make `opened_object` mutable here
            return naming_obj.named_object.as_pipe().and_then(|pipe| {
                let bytes_read = pipe.read(buffer, 0, naming_obj.access_rights)?;
                Ok(bytes_read) // Return the bytes written
            });
        }
        Err(Errno::ENOTSUP)
        
    } else {
        Err(Errno::EACCES)
    }
}

/// Move the object pointer for the named object referenced by `object_handle` to the specified `offset` from the `origin`. \
/// Returns `Ok(nr of bytes seeked)` or `Err`.
pub fn seek(cap: &Capability<NamingObject>, offset: usize, origin: SeekOrigin) -> Result<usize, Errno> {
    if let Some(naming_obj) = cap.invoke() {
        if naming_obj.named_object.is_file() {
            // Make `opened_object` mutable here
            return naming_obj.named_object.as_file().and_then(|file| {
                let new_pos = match origin {
                    SeekOrigin::Start => offset,
                    SeekOrigin::End => file.stat()?.size + offset,
                    SeekOrigin::Current => naming_obj.position.load(Ordering::SeqCst) + offset,
                };
                naming_obj.position.store(new_pos, Ordering::SeqCst);
                Ok(new_pos) // Success
            });
        }
        Err(Errno::ENOTSUP)
    } else {
        Err(Errno::EACCES)
    }
}

/// Close the named object referenced by `object_handle`.
/// Returns `Ok(0)` or `Err(errno)`
/*pub fn close(object_handle: usize) -> Result<usize, Errno> {
    open_objects::close(object_handle)
}
 */

pub fn close(cap: &Capability<NamingObject>) -> Result<usize, Errno> {
    if let Some(obj) = cap.invoke() {
        todo!("Close not yet implemented");
    } else {
        Err(Errno::EACCES)
    }
}

/// Create a directory named 'name' in the directory given by the capability object. \
/// TODO But only if it doesn't already exist
/// Returns `Ok(Capability<NamingObject>)` or `Err(errno)`
pub fn mkdir(name: &str, parent: Capability<NamingObject>, parent_handle: usize) -> Result<Capability<NamingObject>, Errno> {
    // Check rights of parent cap
    if let Some(dir)  = parent.invoke(){
        if dir.access_rights.intersects(OpenOptions::CREATE) {
            return dir.named_object.as_dir().and_then(|directory| {
                if let Ok(obj) = directory.create_dir(name, Mode::new(0)) {
                    Ok(create_naming_capability(obj, OpenOptions::all(), Some(parent_handle)))
                } else { Err(Errno::EACCES) }
            });
        }
    }
    Err(Errno::EACCES)
    
    /*
    // Split the path into components
    let mut components: Vec<&str> = path.split("/").collect();

    // Remove the last component (the name of the new directory)
    let new_dir_name = components.pop();

    // We need parent directory to create the new directory
    let parent_dir = if components.len() == 1 {
        "/".to_string()
    } else {
        components.join("/") // Joins the remaining components
    };

    // Safely lookup the parent directory and create the new file
    let result = lookup::lookup_dir(&parent_dir)
        .and_then(|dir| {
            new_dir_name
                .ok_or(Errno::EINVAL) // Handle missing file name
                .and_then(|name| dir.create_dir(name, Mode::new(0))) // Create the file
        });
        
     */
}

/// Create an empty file defined by `path`. \
/// Returns `Ok(0)` or `Err(errno)`
pub fn touch(name: &str, cap: &Capability<NamingObject>) -> Result<Capability<NamingObject>, Errno> {
    // Verify we have a valid filename
    if name.is_empty() {
        return Err(Errno::EINVAL);
    }

    // Ensure we don't try to process paths with '/' in them since we already have the directory capability
    if name.contains('/') {
        return Err(Errno::EINVAL);
    }

    // Safely lookup the parent directory and create the new file
    if let Some(naming_obj) = cap.invoke() {
        if naming_obj.named_object.is_dir() && naming_obj.access_rights.contains(OpenOptions::CREATE) {
            let result = naming_obj.named_object
                .as_dir()
                .and_then(|dir| dir.create_file(name, Mode::new(0))); // Create the file)
                

            return match result {
                Ok(obj) => Ok(create_naming_capability(obj, OpenOptions::empty(), None)), // Successfully created the file //TODO PARENT
                Err(_) => {
                    // Handle the error here (e.g., logging or returning the error code)
                    Err(Errno::ENOTDIR)
                }
            }
        }
    }
    Err(Errno::EINVAL)
}

/// Read next directory entry of directory referenced by `dir_handle` \
/// Returns: \
///   `Ok(1)` next directory entry in `dentry` \
///   `Ok(0)` no more entries in the directory \
///   `Err`   error code
pub fn readdir(dir_handle: usize, dentry: Option<&mut RawDirent>) -> Result<usize, Errno> {
    todo!(); //TODO even necessary if access is only through caps???
    let res = open_objects::readdir(dir_handle);
    match res {
        Ok(dir_entry) => {
            match dir_entry {
                Some(dir_entry_data) => {
                    // copy data
                    let mut de: RawDirent = RawDirent::new();
                    de.d_type = dir_entry_data.file_type as usize;
                    let name_bytes: &[u8] = dir_entry_data.name.as_bytes();
                    let len = name_bytes.len().min(255); // Avoid overflow
                    de.d_name[..len].copy_from_slice(&name_bytes[..len]);

                    // Write the Dirent structure to the provided dentry pointer
                    if let Some(dentry) = dentry {
                        *dentry = de;
                        Ok(1) // Indicate success
                    } else {
                        Err(Errno::EUNKN) // Handle null pointer case
                    }
                }
                None => Ok(0),
            }
        }
        Err(e) => Err(e),
    }
}

/// Get the current working directory and return path in `buffer`. \
/// Return: `Ok(len of string)` or `Err(errno)`
pub fn cwd(buffer: &mut [u8]) -> Result<usize, Errno> {
    //TODO check if it only returns path and if that is a sec leakage
    // Lock the CWD mutex to access its value
    let cwd = CWD.lock();

    // Get the string as bytes
    let cwd_bytes = cwd.as_bytes();

    // Calculate how much data can be copied (leave space for the null terminator)
    let len_to_copy = (buffer.len() - 1).min(cwd_bytes.len()); // Reserve space for the null terminator

    // Copy the data into the buffer
    buffer[..len_to_copy].copy_from_slice(&cwd_bytes[..len_to_copy]);

    // Add the null terminator if there is space
    if buffer.len() > len_to_copy {
        buffer[len_to_copy] = 0;
    }

    // Return the total length including the null terminator, or just the copied length
    Ok(len_to_copy + 1)
}

///
/// Description: Change working directory \
/// Parameters: `path` absolute path \
/// Return: `Ok(0)` or `Err(errno)`
///
pub fn cd(path: &String) -> Result<usize, Errno> {
    //TODO check security
    let result = lookup::lookup_dir(path);
    match result {
        Ok(_) => {
            let mut cwd = CWD.lock();
            *cwd = path.clone();
            Ok(0)
        }
        Err(_) => {
            // Handle the error here (e.g., logging or returning the error code)
            Err(Errno::ENOTDIR)
        }
    }
}

/// Create a named pipe using `path`. \
/// Returns `Ok(0)` or `Err(errno)`
pub fn mkfifo(path: &str, flags: OpenOptions, cap_to_dir:  &Capability<NamingObject>) -> Result<Capability<NamingObject>, Errno> {
    //Before: mkfifo -> open pipe. But any program could "steal" the pipe if it knows the path. But then access wouldnt be possible... No "Leak" but Still unsafe
    //TODO Need Cap to parent directory, Ensure path is without / and just create with the given cap?? But then addressing only using caps...
    
    // Split the path into components
    let mut components: Vec<&str> = path.split("/").collect();

    // Remove the last component (the name of the new file)
    let new_pipe_name = components.pop();

    // We need parent directory to create the new file
    let parent_dir = if components.len() == 1 {
        "/".to_string()
    } else {
        components.join("/") // Joins the remaining components
    };
    info!("mkfifo: parent dir: {}", parent_dir);
    // Safely lookup the parent directory and create the new pipe
    let result = lookup::lookup_dir(&parent_dir)
        .and_then(|dir| {
            new_pipe_name
                .ok_or(Errno::EINVAL) // Handle missing file name
                .and_then(|name| dir.create_pipe(name, Mode::new(0))) // Create the pipe
        })
        .map(|_| 0); // Convert the success result to 0
    
    match result {
        Ok(_) =>{ // Successfully created the pipe
            // Try to open the pipe
            open_object(path, flags, cap_to_dir)
        }, 
        Err(_) => {
            // Handle the error here (e.g., logging or returning the error code)
            Err(Errno::ENOTDIR)
        }
    }
}


fn open_object(path: &str, flags: OpenOptions, capability_to_dir: &Capability<NamingObject>) -> Result<Capability<NamingObject>, Errno> {
    match open_objects::open(path, flags).or_else(|e| {
        if flags.contains(OpenOptions::CREATE) && e != Errno::EEXIST {
            touch(path, capability_to_dir).and_then(|_| open_objects::open(path, flags)) //TODO touch with Cap handling???
        } else {
            Err(e)
        }
    }) {
        Ok(obj) => {
            info!("opened object at path: {}", path);
            Ok(create_naming_capability(obj, flags, None))
        },
        Err(e) => Err(e),
    }
}