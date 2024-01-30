use std::{net::UdpSocket, time::Duration};

use poggers::structures::process::{implement::utils::ProcessUtils, Process};

#[poggers_derive::create_entry(no_console)]
pub fn main() {
    let sock = UdpSocket::bind("127.0.0.1:29849").unwrap();
    let mut buf = [1; 1];
    sock.connect("127.0.0.1:28713").expect("uanble to connect");

    let proc = Process::this_process();
    let modd = proc.get_base_module().unwrap();

    println!("sending data");
    std::thread::sleep(Duration::from_secs(2));
    sock.send(&buf);
}
