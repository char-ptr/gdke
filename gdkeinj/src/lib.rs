use std::{
    collections::HashMap,
    ffi::c_void,
    mem::size_of,
    net::UdpSocket,
    ptr::{null, slice_from_raw_parts},
    time::Duration,
};

use poggers::{
    structures::process::{implement::utils::ProcessUtils, Process},
    traits::Mem,
};
use retour::static_detour;

// *const i32, *const i32, *const i32, bool
type open_and_parse_t = unsafe extern "fastcall" fn(*const i32, *const i32, *const u8, bool) -> ();
static_detour! {
    pub static OpenAndParse:  unsafe extern "fastcall" fn(*const i32, *const i32, *const u8, bool) -> ();
}
#[repr(u8)]
#[derive(Debug)]
enum SigErrors {
    NotFound,
}
fn find_sig_addr(sig: &str) -> Result<*const c_void, SigErrors> {
    let proc = Process::this_process();
    let modd = proc.get_base_module().unwrap();

    // let sig = SIGS.get(sig_type).ok_or(SigErrors::NotFound)?;
    let addr = modd
        .scan(sig)
        .map_err(|_| SigErrors::NotFound)?
        .ok_or(SigErrors::NotFound)? as isize;
    let ptr_to_fn = (addr as usize + size_of::<u8>()) as *const u8;
    let mut addr_offset = [0; 4];
    unsafe { std::ptr::copy(ptr_to_fn, addr_offset.as_mut_ptr(), 4) };
    let by = i32::from_ne_bytes(addr_offset);
    let fn_ptr = (addr + by as isize + 5) as *const c_void;
    println!(
        "fnptr = {:x?} & {} B = ${addr_offset:?}, ${by:?} dede {addr:x?}",
        fn_ptr, fn_ptr as isize
    );

    Ok(fn_ptr)
}
#[cfg_attr(debug_assertions, poggers_derive::create_entry(no_free))]
#[cfg_attr(not(debug_assertions), poggers_derive::create_entry(no_console))]
pub fn main() {
    let sock = UdpSocket::bind("127.0.0.1:29849").unwrap();
    sock.connect("127.0.0.1:28713").expect("uanble to connect");

    println!("sending data, waiting for sig ver");
    let buf = [];
    sock.send(&buf).ok();

    let mut capy = vec![0u8; 256];
    sock.recv(&mut capy).unwrap();
    let mut sizer = [0; 8];
    sizer.copy_from_slice(&capy[..8]);
    let sizer_usize = usize::from_ne_bytes(sizer);
    let content = &capy[std::mem::size_of::<usize>()..];
    let string_content = String::from_utf8_lossy(content);
    let str_content = &string_content[..sizer_usize];
    let fn_ptr = find_sig_addr(str_content);
    let fn_ptr = match fn_ptr {
        Ok(x) => x,
        Err(err) => {
            println!("err  {err:?}");

            std::thread::sleep(Duration::from_secs(100));
            // sock.send(&[err as u8]).ok();
            return;
        }
    };

    println!("hooking fnptr");
    let sock2 = sock.try_clone().unwrap();
    unsafe {
        let open_and_parse = std::mem::transmute::<isize, open_and_parse_t>(fn_ptr as isize);
        let opp = OpenAndParse
            .initialize(open_and_parse, move |_, _, key, _| {
                println!("hook has been called");
                let ptr_to_key = (key as usize + 8) as *const *const u8;
                println!("key ptr = {:p}", ptr_to_key);
                #[cfg(debug_assertions)]
                {
                    println!("[debug] waiting for input");
                    std::io::stdin().read_line(&mut String::new());
                }
                sock2.send(&*slice_from_raw_parts(*ptr_to_key, 32)).unwrap();
                std::thread::sleep(Duration::from_secs(1000))
                // panic!("good ridance.")
            })
            .unwrap();
        opp.enable().expect("failed to enable detour");
        println!("detour enabled {}", opp.is_enabled());
    }
    sock.send(&[0, 0, 0, 0]).ok();
}
