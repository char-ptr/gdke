use poggers::mem::{external::process::ExProcess, traits::Mem};

const SIGS: [&str; 3] = ["48 8D 05 ? ? ? ? 41 8A 04 04","48 8D 05 ? ? ? ? 0F B6 ? 03","4C 8D 05 ? ? ? ? 0F 1F 40 00"];

fn main() {
    let proc = ExProcess::new_from_name("SoundSpacePlus.exe".to_string()).unwrap();
    let bm = proc.get_base_module().unwrap();
    for sig in SIGS.iter() {
        let res = unsafe {bm.scan_virtual(sig)};
        if let Some(x) = res {
            let data = unsafe {bm.resolve_relative_ptr(x+3, 4)};
            if let Ok(x) = data {
                println!("found key @ {:X}", x);
                let key_data = unsafe {bm.read_sized(x, 32)};
                if let Ok(x) = key_data {
                    print!("Key: ");
                    for i in x {
                        print!("{:02X}", i);
                    }
                }
            } else {
                println!("Unable to resolve lea relative ptr");
            }
            // println!("Found sig: {:X}", x);
        } else {
            println!("Failed to find with sig: {}", sig);
        }
    }
}
