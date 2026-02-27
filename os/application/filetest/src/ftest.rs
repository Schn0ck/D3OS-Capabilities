#![no_std]
extern crate alloc;

use naming::ROOT;
use naming::shared_types::{OpenOptions, SeekOrigin};
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
// 
// 
// /// Check if a already created file cannot be created again
// fn multiple_create() {
//     print!("   test: multiple creates - ");
//     let ret = naming::open("/test.txt", OpenOptions::CREATE | OpenOptions::READWRITE, ROOT);
//     match ret {
//         Ok(fd) => {
//             let ret2 = naming::open("/test.txt", OpenOptions::CREATE | OpenOptions::READWRITE, ROOT);
//             match ret2 {
//                 Ok(fd) => {
//                     println!("failed second create");
//                 }
//                 Err(e) => {
//                     println!("ok");
//                 }
//             }
//         }
//         Err(e) => {
//             println!("failed first create {:?}", e);
//         }
//     }
// }
// 
// 
// 
#[unsafe(no_mangle)]
pub fn main() {
//     println!("naming tests");
// 
//     // opening file
//     let res = naming::open("/file.txt", OpenOptions::READWRITE | OpenOptions::CREATE, ROOT);
//     if res.is_err() {
//         println!("open error = {:?}", res);
//         return;
//     }
//     let cap_handle = res.unwrap();
//     println!("open file '/file.txt', cap_handle = {:?}", cap_handle);
// 
//     // writing to file
//     let buff = "Hello, World!".as_bytes();
//     let res = naming::write(cap_handle, buff);
//     println!("write result = {:?}", res);
// 
//     // writing to file again
//     let buff2 = " NRW Duesseldorf.".as_bytes();
//     let res = naming::write(cap_handle, buff2);
//     println!("write result = {:?}", res);
// 
//     // seek to beginning
//     let res = naming::seek(cap_handle, 0, SeekOrigin::Start); //todo
//     println!("seek result = {:?}", res);
// 
//     // reading from file
//     let mut rbuff: [u8; 512] = [0; 512];
//     let res = naming::read(cap_handle, &mut rbuff);
//     println!("read result = {:?}", res);
//     if let Ok(len) = res {
//         for (i, byte) in rbuff.iter().enumerate() {
//             if i >= len {
//                 break;
//             }
//             if byte.is_ascii_graphic() || *byte == b' ' {
//                 print!("{}", *byte as char);
//             } else {
//                 print!(".");
//             }
//         }
//     }
//     println!("");
// 
//     //let close_res = naming::close(cap_handle);
//     //println!("close result = {:?}", close_res);
// 
//     let res = naming::mkdir("/test", ROOT);
//     println!("created dir '/test' = {:?}", res);
// 
//     let res = naming::mkdir("/test/dir1", ROOT);
//     println!("created dir '/test/dir1' = {:?}", res);
// 
//     let res = naming::mkdir("/test/dir2", ROOT);
//     println!("created dir '/test/dir2' = {:?}", res);
// 
//     let res = naming::touch("/test/file1.txt", ROOT);
//     println!("created file '/test/file1.txt' = {:?}", res);
// 
//     // opening directory
//     let res = naming::open("/test", OpenOptions::DIRECTORY, ROOT);
//     if res.is_err() {
//         println!("open error = {:?}", res);
//         return;
//     }
//     let fd = res.unwrap();
//     println!("open dir '/test'");
// 
//     loop {
//         let res = naming::readdir(fd);
//         match res {
//             Ok(data) => {
//                 match data {
//                     Some(content) => println!("   readdir data = {:?}", content),
//                     None => break,
//                 }
//             },
//             Err(_) => { 
//                 println!("   readdir failed");
//                 break;
//             },
//         }
//     }
// 
//     //let close_res = naming::close(fd);
//     //println!("close result = {:?}", close_res);
// 
//     println!("naming test: end");
}
//TODO make it work, use touch. then open