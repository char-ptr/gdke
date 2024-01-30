#![feature(offset_of)]
use std::{
    error::Error,
    ffi::{c_void, CStr, CString},
    mem::{size_of, transmute},
    ptr::{addr_of, null, null_mut},
    time::Duration,
};

use dll_syringe::{process::OwnedProcess, Syringe};
use poggers::{structures::process::Process, traits::Mem};
use windows::Win32::System::{
    Diagnostics::Debug::{GetThreadContext, CONTEXT, IMAGE_NT_HEADERS64},
    Threading::{ResumeThread, SuspendThread},
};
use windows::{
    core::{PCSTR, PSTR},
    Win32::{
        Foundation::{BOOL, HINSTANCE},
        System::{
            ProcessStatus::{K32GetModuleInformation, MODULEINFO},
            SystemServices::IMAGE_DOS_HEADER,
            Threading::{
                CreateProcessA, NtQueryInformationProcess, ProcessBasicInformation,
                TerminateProcess, CREATE_SUSPENDED, PEB, PROCESS_BASIC_INFORMATION,
                PROCESS_INFORMATION, STARTUPINFOA,
            },
        },
    },
};

fn create_pstr(c_str: &CStr) -> PSTR {
    PSTR::from_raw(c_str.as_ptr() as *mut u8)
}

pub unsafe fn spawn_and_inject(proc: &str) {
    let cmd_line_c = CString::new(proc).expect("invalid cstr");
    let start_up_info = STARTUPINFOA {
        ..Default::default()
    };
    let mut proc_info = PROCESS_INFORMATION {
        ..Default::default()
    };
    let mod_name = PCSTR::null();
    CreateProcessA(
        mod_name,
        create_pstr(cmd_line_c.as_c_str()),
        None,
        None,
        BOOL(0),
        CREATE_SUSPENDED,
        None,
        mod_name,
        &start_up_info,
        &mut proc_info,
    );
    // patch entry point...
    let mut ptr_to_pbi: PROCESS_BASIC_INFORMATION = std::mem::zeroed();

    let stat = NtQueryInformationProcess(
        proc_info.hProcess,
        ProcessBasicInformation,
        &mut ptr_to_pbi as *mut _ as *mut c_void,
        size_of::<PROCESS_BASIC_INFORMATION>() as u32,
        &mut 0,
    );
    let proc = Process::find_pid(proc_info.dwProcessId).unwrap();
    let pebby: PEB = proc.read(ptr_to_pbi.PebBaseAddress as usize).expect("the");
    let pImage = pebby.Reserved3[1] as usize;
    let e_lf: u32 = proc
        .read(pImage + std::mem::offset_of!(IMAGE_DOS_HEADER, e_lfanew))
        .expect("bruh");
    let entry: u32 = proc
        .read(
            pImage
                + e_lf as usize
                + std::mem::offset_of!(IMAGE_NT_HEADERS64, OptionalHeader.AddressOfEntryPoint),
        )
        .expect("bruh");
    let entry = pImage + entry as usize;
    println!("entry = {:x}", entry);
    let entry_insts: [u8; 2] = proc.read(entry).expect("failed to read entry");
    let pay_load: [u8; 2] = [0xEB, 0xFE];
    proc.write(entry, &pay_load);
    println!("{:x?}", entry_insts);
    //
    // resume the thread
    ResumeThread(proc_info.hThread);
    // wait until trapped... and inject
    let target = OwnedProcess::from_pid(proc.get_pid()).unwrap();
    let syrnge = Syringe::for_process(target);
    let injmod = syrnge.inject("./gdkeinj.dll").unwrap();

    // we're done. let's kill the process.
    println!("waiting 2secs ");
    std::thread::sleep(Duration::from_secs(2));
    println!("waited 2secs, restoring..",);
    println!("{:?}", injmod.handle());
    proc.write(entry, &entry_insts);
    // TerminateProcess(proc_info.hProcess, 1);
}
