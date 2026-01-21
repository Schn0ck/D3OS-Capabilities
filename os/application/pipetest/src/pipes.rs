#![no_std]

extern crate alloc;

use naming::shared_types::OpenOptions;
use naming::{close, mkfifo, open, read, write, ROOT, SHARED_PIPE};

use concurrent::thread;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use capabilities::*;
use terminal::write::print;

const PIPE: &str = "/mypipe";
const NR_OF_ITERATIONS: u32 = 6;

fn writer_thread() {
    thread::sleep(2000); //wait for reader to be ready
    println!("---writer_thread: start");
    let thread = thread::current().unwrap();
    let mut buff= [0;1];
    //let res = read(SHARED_PIPE, &mut buff);
    let cap_handle = 2; //buff[0] as usize; //receive the cap number
    
    
    println!("---writer_thread: got capability handle = {}", cap_handle);

    let mut cnt = 0;
    let mut wbuff: [u8; 1] = [0; 1];
    let mut ch: u8 = b'A'; // start at ASCII 'A'
    loop {
        wbuff[0] = ch;
        let res = write(cap_handle, &wbuff); 

        if res.is_err() {
            println!("---writer_thread: write failed, error: {:?}", res);
        } else {
            println!("---writer_thread: wrote one byte = '{}'", ch as char);

            // Next letter
            ch = if ch == b'Z' {
                b'A' // wrap around after 'Z'
            } else {
                ch + 1
            };
        }
        cnt = cnt + 1;
        if cnt > NR_OF_ITERATIONS {
            break;
        }
       concurrent::thread::sleep(1000);
    }

    // close(cap_handle);
    println!("---writer_thread: end");
}

fn reader_thread() {
    // let thread = thread::current().unwrap();
    // println!("reader_thread (tid={}): start", thread.id());
    // let res = open("/mypipe", OpenOptions::READONLY);
    // if res.is_err() {
    //     println!("reader_thread: open failed, error: {:?}", res);
    //     return;
    // }
    let mut buff= [0;1];
    //let res = read(SHARED_PIPE, &mut buff);
    let cap_handle = 2; // buff[0] as usize; //receive the cap number


    let mut rbuff: [u8; 1] = [0; 1];
    let mut cnt = 0;
    loop {
        let res = read(cap_handle, &mut rbuff);
        if res.is_err() {
            println!("+++reader_thread: read failed, error: {:?}", res);
        } else {
            if rbuff[0].is_ascii() {
                let ch = rbuff[0] as char;
                println!("+++reader_thread: read one byte '{}', read = {}", ch, res.unwrap());
            } else {
                println!("+++reader_thread: read invalid data");
            }
        }
        cnt = cnt + 1;
        if cnt > NR_OF_ITERATIONS {
            break;
        }
       concurrent::thread::sleep(1000);
    }

    // close(cap_handle);
    println!("+++reader_thread: end");
}

#[unsafe(no_mangle)]
pub fn main() {
    println!("named pipe demo: start");

    println!("got root capability");

    // debug_print_caps(thread::current().unwrap().id());

    let res = mkfifo("/mypipe", OpenOptions::READWRITE | OpenOptions::SHARE, ROOT);
    if res.is_err() {
        println!("mkfifo failed, error: {:?}", res);
        return;
    }
    let pipe_cap = res.unwrap();
    println!("mkfifo: ok, cap_handle = {}", pipe_cap);

    write(pipe_cap, b"Hello from main thread!").unwrap();
    let buf = &mut [0u8; 23];
    read(pipe_cap, buf).unwrap();

    // Print individual bytes as characters
    print!("Read: ");
    for &byte in buf.iter() {
        if byte != 0 {  // Skip null bytes
            print!("{}", byte as char);
        }
    }
    println!("");


    share_naming_object(thread::current().unwrap().id(), pipe_cap);

    // 
    // let mut buff: [u8; 1] = [0; 1];
    // buff[0] = b'A';
    // let result = write(pipe_cap, &buff);
    // let mut rbuff: [u8; 1] = [0; 1];
    // let result2 = read(pipe_cap, &mut rbuff);
    // 
    // println!("read result = {}", rbuff[0] as char);
    // 
    //Ok up to here

    // debug_print_caps(thread::current().unwrap().id());
    
    let writer = thread::create(|| {
        writer_thread();
    });
    
    if let Some(w) = writer {
        println!("Starting writer, id {}", w.id());
        let num = share_naming_object(w.id(),pipe_cap); //TODO share cap with custom rights (e.g. readonly on a readwrite cap)
        println!("Shared pipe cap {} with writer thread {}", num, w.id());
        let buff= [num as u8];
        let res = write(SHARED_PIPE, &buff); 
        println!("Sent pipe cap to writer thread {}", w.id());
        w.join()
    }

    println!("Writer done, starting reader");
    
    let reader = thread::create(|| {
        reader_thread();
    });
    
    if let Some(r) = reader {
        let num = share_naming_object(r.id(),pipe_cap);
        let buff= [num as u8];
        let res = write(SHARED_PIPE, &buff);
        r.join();
    }

    println!("named pipe demo: done");
}
