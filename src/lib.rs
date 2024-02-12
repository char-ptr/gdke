pub mod versioning;
use std::{
    ffi::{c_void, CStr, CString},
    io::Write,
    mem::{size_of, transmute},
    net::UdpSocket,
    path::Path,
};

use dll_syringe::{process::OwnedProcess, Syringe};
use poggers::{exports::HANDLE, structures::process::Process, traits::Mem};
use rust_embed::RustEmbed;
use thiserror::Error;
use windows::{
    core::{PCSTR, PSTR},
    Win32::{
        Foundation::BOOL,
        System::{
            SystemServices::IMAGE_DOS_HEADER,
            Threading::{
                CreateProcessA, TerminateProcess, CREATE_SUSPENDED, PROCESS_BASIC_INFORMATION,
                PROCESS_INFORMATION, STARTUPINFOA,
            },
        },
    },
};
use windows::{
    Wdk::System::Threading::{NtQueryInformationProcess, ProcessBasicInformation},
    Win32::System::{Diagnostics::Debug::IMAGE_NT_HEADERS64, Threading::ResumeThread},
};

use crate::versioning::check_gd_ver;
#[repr(u8)]
#[derive(Error, Debug)]
enum SigErrors {
    #[error("Signature not found")]
    NotFound,
}
impl From<u8> for SigErrors {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NotFound,
            default => Self::NotFound,
        }
    }
}

fn create_pstr(c_str: &CStr) -> PSTR {
    PSTR::from_raw(c_str.as_ptr() as *mut u8)
}
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/target/release"]
#[include = "gdkeinj.dll"]
struct GdkeInj;

struct ProcKiller(HANDLE);
impl Drop for ProcKiller {
    fn drop(&mut self) {
        unsafe {
            TerminateProcess(self.0, 0).ok();
        }
    }
}
pub unsafe fn spawn_and_inject(proc: &str) -> anyhow::Result<[u8; 32]> {
    let pth = Path::new(proc);
    if !pth.is_file() {
        panic!("file does not exist");
    }
    let cmd_line_c = CString::new(proc).expect("invalid cstr");
    let start_up_info = STARTUPINFOA {
        wShowWindow: 0,
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
    )?;
    // patch entry point...
    let mut ptr_to_pbi: PROCESS_BASIC_INFORMATION = std::mem::zeroed();

    NtQueryInformationProcess(
        proc_info.hProcess,
        ProcessBasicInformation,
        &mut ptr_to_pbi as *mut _ as *mut c_void,
        size_of::<PROCESS_BASIC_INFORMATION>() as u32,
        &mut 0,
    );
    let _pkr = ProcKiller(proc_info.hProcess);
    let proc = Process::find_pid(proc_info.dwProcessId).unwrap();
    let image_base_addr: *const c_void = proc
        .read(ptr_to_pbi.PebBaseAddress as usize + 0x10)
        .expect("the");
    let mut headers = [0; 4096];
    proc.raw_read(image_base_addr as usize, headers.as_mut_ptr(), 4096)?;
    let dos_hdr = transmute::<*const u8, *const IMAGE_DOS_HEADER>(headers.as_ptr());
    let nt_hdrs = transmute::<*const u8, *const IMAGE_NT_HEADERS64>(
        headers
            .as_ptr()
            .wrapping_add((*dos_hdr).e_lfanew.try_into().unwrap()),
    );
    let code_entry =
        image_base_addr.wrapping_add((*nt_hdrs).OptionalHeader.AddressOfEntryPoint as usize);
    println!("entry = {:p}", code_entry,);
    let entry_insts: [u8; 2] = proc
        .read(code_entry as usize)
        .expect("failed to read entry");
    let pay_load: [u8; 2] = [0xEB, 0xFE];
    proc.write(code_entry as usize, &pay_load)?;
    //
    // resume the thread
    ResumeThread(proc_info.hThread);
    // wait until trapped... and inject
    let sock = UdpSocket::bind("127.0.0.1:28713").expect("failed to bind socket");
    let res: anyhow::Result<()> = {
        let target = OwnedProcess::from_pid(proc.get_pid()).unwrap();
        let syrnge = Syringe::for_process(target);
        let dll_loc = if cfg!(debug_assertions) {
            String::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/target/debug/gdkeinj.dll"
            ))
        } else {
            let gdke_inj_dll =
                GdkeInj::get("gdkeinj.dll").expect("failed to get dll from embeded resources");
            let tmp = std::env::temp_dir();
            let loc = tmp.join("gdkeinj.dll");
            let mut file = std::fs::File::create(&loc).unwrap();
            file.write_all(&gdke_inj_dll.data).unwrap();
            loc.to_str().map(|x| x.to_string()).unwrap()
        };
        let game_ver = check_gd_ver(pth)?;
        println!("gamever = {game_ver}");
        let sig_id = match game_ver
            .chars()
            .next()
            .ok_or(anyhow::anyhow!("unable to check gd version"))?
        {
            '4' => 0u32,
            '3' => 1u32,
            _ => return Err(anyhow::anyhow!("invalid godot version")),
        };

        println!("injecting dll ({})", dll_loc);
        syrnge.inject(dll_loc)?;

        println!("waiting until udp is ok ");

        let (_, addr) = sock.recv_from(&mut [0]).unwrap();
        println!("using sig id {sig_id}");
        sock.send_to(&sig_id.to_ne_bytes(), addr).unwrap();
        let mut error = [0u8; 4];
        sock.recv(&mut error)?;
        println!("errors -> {error:?}");
        if error.is_empty() {
            return Err(SigErrors::from(error[0]).into());
        }
        Ok(())
    };
    res?;
    // we're done. let's kill the process.
    println!("done, running code",);
    #[cfg(debug_assertions)]
    {
        println!("[debug] waiting for input");
        std::io::stdin().read_line(&mut String::new());
    }
    proc.write(code_entry as usize, &entry_insts)?;
    println!("waiting for call.");
    let mut key = [0; 32];
    sock.recv(&mut key)?;
    if key.len() == 1 {
        eprintln!("recieved err");
        return Err(SigErrors::from(key[0]).into());
    }
    println!("recieved key, term");
    Ok(key)
}
