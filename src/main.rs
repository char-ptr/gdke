// use poggers::{external::process::ExProcess, traits::Mem};

use poggers::structures::process::Process;

fn main() {
    let mut pid = 0;
    {
        let proc = Process::find_by_name("pog.exe").unwrap();
        pid = proc.get_pid();
    }

    let key = gdke::get_from_pid(pid).expect("unable to find key");

    println!("Key: {}", key);
}
