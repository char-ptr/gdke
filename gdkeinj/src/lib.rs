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

#[cfg_attr(debug_assertions, poggers_derive::create_entry)]
#[cfg_attr(not(debug_assertions), poggers_derive::create_entry(no_console))]
pub fn main() {
    let mut sigs = HashMap::<u32, (&'static str, i32)>::new();
    sigs.insert(
        1,
        ("E8 ? ? ? ? 85 C0 0F 84 ? ? ? ? 49 8B 8C 24 ? ? ? ?", -0x3c),
    );
    let sock = UdpSocket::bind("127.0.0.1:29849").unwrap();
    sock.connect("127.0.0.1:28713").expect("uanble to connect");

    let proc = Process::this_process();
    let modd = proc.get_base_module().unwrap();

    println!("sending data, waiting for sig ver");
    let buf = [1; 1];
    sock.send(&buf).ok();

    let mut sig_type = [0; 4];
    sock.recv(&mut sig_type).unwrap();
    let int_sig = u32::from_ne_bytes(sig_type);
    let sig = sigs.get(&int_sig).expect("sig type match not compatible");
    let addr = modd.scan(sig.0).unwrap().unwrap() as isize;
    let ptr_to_fn = (addr as usize + size_of::<u8>()) as *const u8;
    let mut addr_offset = [0; 4];
    unsafe { std::ptr::copy(ptr_to_fn, addr_offset.as_mut_ptr(), 4) };
    let by = i32::from_ne_bytes(addr_offset);
    let fn_ptr = (addr + by as isize + 5) as *const c_void;
    println!("fnptr = {:x?}", fn_ptr);

    println!("sig found: {:x} ", addr);
    let sock2 = sock.try_clone().unwrap();
    unsafe {
        let open_and_parse = std::mem::transmute::<isize, open_and_parse_t>(fn_ptr as isize);
        let opp = OpenAndParse
            .initialize(open_and_parse, move |_, _, key, _| {
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
    sock.send(&[]).ok();
}
