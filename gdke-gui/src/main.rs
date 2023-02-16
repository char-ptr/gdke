#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::error::Error;

use gdke_gui::{app::gdkeApp, Data};
use poggers::{external::process::ExProcess, traits::Mem};

const SIGS: [&str; 3] = ["48 8D 05 ? ? ? ? 41 8A 04 04","48 8D 05 ? ? ? ? 0F B6 ? 03","4C 8D 05 ? ? ? ? 0F 1F 40 00"];

fn main() {

    let (stx, srx) = std::sync::mpsc::channel::<Data>();
    let (ctx, crx) = std::sync::mpsc::channel::<Data>();


    let jh = std::thread::spawn(move|| {
        loop {
            if let Ok(x) = crx.try_recv() {
                match x {
                    Data::Pid(pid) => {
                        println!("Got pid: {}", pid);
                        match (|| -> Result<(),Box<dyn Error>> {

                            let proc = ExProcess::new_from_pid(pid)?;
                            let bm = proc.get_base_module()?;
                            for sig in SIGS.iter() {
                                let res = unsafe {bm.scan_virtual(sig)};
                                if let Some(x) = res {
                                    let data = unsafe {bm.resolve_relative_ptr(x+3, 4)};
                                    if let Ok(x) = data {
                                        println!("found key @ {:X}", x);
                                        let key_data = unsafe {bm.read_sized(x, 32)};
                                        if let Ok(x) = key_data {
                                            // print!("Key: ");
                                            let mut data_string = String::new();
                                            for i in x {
                                                data_string.push_str(&format!("{:02X}", i));
                                            }
                                            println!("sending data {}", data_string);
                                            stx.send(Data::Key(data_string)).unwrap();
                                            return Ok(());
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

                        })() {
                            Ok(_) => {},
                            Err(er) => {
                                println!("Error: {}", er);
                                stx.send(Data::Failure(er.to_string())).unwrap();
                                continue;
                            }
                        }
                    },
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "gdke",
        native_options,
        Box::new(move |cc| Box::new(gdkeApp::new(cc, srx, ctx))),
    );

    jh.join();
}
