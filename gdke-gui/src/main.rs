#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::error::Error;

use gdke_gui::{app::gdkeApp, Data};
use poggers::{external::process::ExProcess, traits::Mem};

fn main() {
    let (stx, srx) = std::sync::mpsc::channel::<Data>();
    let (ctx, crx) = std::sync::mpsc::channel::<Data>();

    let jh = std::thread::spawn(move || {
        loop {
            if let Ok(x) = crx.try_recv() {
                match x {
                    Data::Pid(pid) => {
                        println!("Got pid: {}", pid);
                        match (|| -> Result<(), Box<dyn Error>> {
                            let key = gdke::get_from_pid(pid)?;
                            stx.send(Data::Key(key)).unwrap();
                            Ok(())
                            // Err("Failed to find key".into())
                        })() {
                            Ok(_) => {}
                            Err(er) => {
                                println!("Error: {}", er);
                                stx.send(Data::Failure(er.to_string())).unwrap();
                                continue;
                            }
                        }
                    }
                    Data::EXIT => {
                        break;
                    },
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    let native_options = eframe::NativeOptions::default();
    let ctx2 = ctx.clone();
    eframe::run_native(
        "gdke",
        native_options,
        Box::new(move |cc| Box::new(gdkeApp::new(cc, srx, ctx2))),
    );
    ctx.send(Data::EXIT).unwrap();

    jh.join();
}
