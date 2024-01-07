use std::error::Error;

use poggers::{
    structures::{
        modules::Module,
        process::{implement::utils::ProcessUtils, External, Process},
    },
    traits::{Mem, MemError},
};
/// # Safety
/// this is a private crate so stooop clippy
pub unsafe fn resolve_relative_ptr(
    proc: &Module<Process<External>>,
    addr: usize,
    offset: usize,
) -> Result<usize, MemError> {
    let real_offset = proc.get_owner().read::<u32>(addr)?;
    println!("Real offset: {:X?}", real_offset);
    let rel = (addr - proc.get_base_address()) + offset;
    let real = rel + real_offset as usize;
    println!("Real: {:X?}", real);
    Ok(proc.get_base_address() + real)
    // Err(anyhow!("lazy"))
}
pub const SIGS: [&str; 6] = [
    "BA 20 00 00 00 4C 89 ? ? 48 8D", // GD 4.2 (from master branch)
    "48 8D 1D ? ? ? ? 4C 8D 2D ? ? ? ? 48 8D 35", // godot 4.0.0
    "48 8D 3D ? ? ? ? 48 85 C0 74 3B",
    "48 8D 05 ? ? ? ? 41 8A 04 04",
    "48 8D 05 ? ? ? ? 0F B6 ? 03",
    "4C 8D 05 ? ? ? ? 0F 1F 40 00",
];

pub fn get_from_pid(pid: u32) -> Result<String, Box<dyn Error>> {
    let proc = Process::find_pid(pid)?;
    let bm = proc.get_base_module()?;
    // println!("Base module: {:X?}", bm);
    for sig in &SIGS {
        let timer = std::time::Instant::now();
        let res = bm.scan(sig);
        println!("Scan took: {}ms", timer.elapsed().as_millis());
        if let Ok(Some(x)) = res {
            println!("hey! something was found!");
            let data = unsafe { resolve_relative_ptr(&bm, x + 3, 4) };
            if let Ok(x) = data {
                println!("found key @ {:X}", x);
                let mut key_data = [0u8; 32];
                if unsafe { proc.raw_read(x, &mut key_data as *mut u8, 32) }.is_ok()
                    && !key_data.is_empty()
                {
                    let mut data_string = String::new();
                    for i in &key_data[..] {
                        data_string.push_str(&format!("{:02X}", i));
                    }
                    return Ok(data_string);
                }
            } else {
                return Err("Unable to resolve lea relative ptr".into());
            }
            // println!("Found sig: {:X}", x);
        } else {
            println!("Failed to find with sig: {}", sig);
            // return Err("Failed to find with sig".into());
        }
    }
    // Ok(())
    Err("Failed to find key".into())
}

