use std::{
    collections::HashMap, ffi::c_void, mem::size_of, net::UdpSocket, ptr::null, time::Duration,
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

const SIGS: [&str; 5] = [
    // call into open_and_parse
    "E8 ? ? ? ? 85 C0 0F 84 ?  ?  ? ? 49 8B 8C 24 ?  ?  ?  ?", // 4.x (4.2.1)
    "E8 ? ? ? ? 89 44 24 50 83 7C 24 ? ?  0F 84 ?  ?  ?  ?  48 8B 44 24 ?", // 3.5.1
    "E8 ? ? ? ? 89 44 24 50 83 7C 24 ? ?  0F 84 ?  ?  ?  ?  48 8B 44 24 ?", // 3.5.1
    "E8 ? ? ? ? 8B D8 85 C0 0F 84 ? ? ?  ?  49 8B 04 24",      // 3.x
    "E8 ? ? ? ? 48 8B 4C 24 ? 89 C5 48 85 C9",                 // 4.3
];
#[repr(u8)]
#[derive(Debug)]
enum SigErrors {
    NotFound,
}
fn find_sig_addr(sig_type: usize) -> Result<*const c_void, SigErrors> {
    let proc = Process::this_process();
    let modd = proc.get_base_module().unwrap();

    let sig = SIGS.get(sig_type).ok_or(SigErrors::NotFound)?;
    let addr = modd
        .scan(sig)
        .map_err(|_| SigErrors::NotFound)?
        .ok_or(SigErrors::NotFound)? as isize;
    let ptr_to_fn = (addr as usize + 1) as *const u8;
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

    let mut sig_type = [0; 4];
    sock.recv(&mut sig_type).unwrap();
    println!("received sig type: {:?}", sig_type);
    let int_sig = u32::from_ne_bytes(sig_type);
    let fn_ptr = find_sig_addr(int_sig as usize);
    let fn_ptr = match fn_ptr {
        Ok(x) => x,
        Err(err) => {
            println!("err  {err:?}");

            std::thread::sleep(Duration::from_secs(100));
            sock.send(&[err as u8]).ok();
            return;
        }
    };

    println!("sending fnptr");
    let sock2 = sock.try_clone().unwrap();
    unsafe {
        let open_and_parse = std::mem::transmute::<isize, open_and_parse_t>(fn_ptr as isize);
        let opp = OpenAndParse
            .initialize(open_and_parse, move |_, _, key, _| {
                println!("hook has been called");
                let mut read_key = [0u8; 32];
                let ptr_to_key = (key as usize + 8) as *const *const u8;
                std::ptr::copy(*ptr_to_key, read_key.as_mut_ptr(), 32);
                sock2.send(read_key.as_slice()).unwrap();
                std::thread::sleep(Duration::from_secs(1000))
                // panic!("good ridance.")
            })
            .unwrap();
        opp.enable().expect("failed to enable detour");
        println!("detour enabled {}", opp.is_enabled());
    }
    sock.send(&(400195u32.to_ne_bytes())).ok();
}
