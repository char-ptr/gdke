use std::{net::UdpSocket, time::Duration};

#[poggers_derive::create_entry(no_free)]
pub fn main() {
    let sock = UdpSocket::bind("127.0.0.1:29849").unwrap();
    let mut buf = [1; 1];
    sock.connect("127.0.0.1:28713").expect("uanble to connect");

    println!("sending data");
    std::thread::sleep(Duration::from_secs(2));
    sock.send(&buf);
}
